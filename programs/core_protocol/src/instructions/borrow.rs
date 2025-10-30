use crate::constants::*;
use crate::errors::*;
// use crate::initialization::loan;
use crate::instructions::helper::*;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    self, Mint, TokenAccount, TokenInterface, TransferChecked,
};
use pyth_solana_receiver_sdk::price_update::{PriceUpdateV2,get_feed_id_from_hex};




#[derive(Accounts)]
pub struct Borrow<'info>{

    // borrower
    #[account(mut)]
    pub borrower: Signer<'info>,

    pub collateral_mint: InterfaceAccount<'info, Mint>,
    pub borrow_mint: InterfaceAccount<'info, Mint>,

    // protocol state to update the total borrows
    #[account(
        mut,
        seeds = [b"protocol_state"],
        bump
    )]
    pub protocol_state: Box<Account<'info, ProtocolState>>,

    // Collateral market (e.g., USDC) for updating  
    #[account(
        mut,
        seeds = [b"market", collateral_mint.key().as_ref()],
        bump
    )]
    pub collateral_market: Box<Account<'info, Market>>,

    #[account(
        mut,
        seeds = [b"market", borrow_mint.key().as_ref()],
        bump ,
        constraint = !borrow_market.paused @ LendingError::MarketPaused,
    )]
    pub borrow_market: Box<Account<'info, Market>>,

    #[account(
        mut,
        seeds = [b"user_account", borrower.key().as_ref(), collateral_market.key().as_ref()],
        bump,
    )]
    pub collateral_position: Box<Account<'info, UserPosition>>,
    
    // #[account(
    //     mut,
    //     seeds = [b"supply_vault", market.key().as_ref()],
    //     bump
    // )]
    // pub borrow_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = borrower,
        space = ANCHOR_DISCRIMINATOR_SIZE + Loan::LEN,
        seeds = [b"loan", collateral_market.key().as_ref(), borrow_market.key().as_ref(), borrower.key().as_ref()],
        bump,
    )]
    pub loan: Box<Account<'info, Loan>>,

    pub token_program: Interface<'info, TokenInterface>,
    pub price_update_col: Account<'info, PriceUpdateV2>,
    pub price_update_borrow: Account<'info, PriceUpdateV2>,
    pub system_program: Program<'info, System>,
}



pub fn borrow_handler(
    ctx: Context<Borrow>,
    shares_amount: u64,
    borrow_amount: u64,
) -> Result<u64> {
    // Implement the borrow logic here

    let collateral_market = &mut ctx.accounts.collateral_market;
    let borrow_market = &mut ctx.accounts.borrow_market;
    let collateral_position = &mut ctx.accounts.collateral_position;
    let market_protocol_state = &mut ctx.accounts.protocol_state;
    let clock = Clock::get()?;

    // ========================================================================
    // STEP 1: Validate loan amount
    // ========================================================================
    require!(
        borrow_amount >= borrow_market.min_borrow_amount,
        LendingError::BorrowTooSmall
    );
    require!(
        borrow_amount <= borrow_market.max_borrow_amount,
        LendingError::BorrowTooLarge
    );

    // ========================================================================
    // STEP 2: Check user has enough FREE rTokens for collateral
    // ========================================================================
    let free_rtokens = collateral_position.free_rtokens();
    require!(
        shares_amount <= free_rtokens,
        LendingError::InsufficientCollateral
    );

    // ========================================================================
    // STEP 3: Accrue interest on both markets
    // ========================================================================
    accrue_interest(collateral_market, clock.unix_timestamp)?;
    accrue_interest(borrow_market, clock.unix_timestamp)?;

    // ========================================================================
    // STEP 4: Convert rTokens to underlying collateral value
    // ========================================================================
    let collateral_mint_supply = collateral_market.total_deposits; 
    let collateral_rtokens_supply = collateral_market.total_deposited_shares;
    
    let underlying_collateral = (shares_amount as u128)
        .checked_mul(collateral_rtokens_supply as u128)
        .ok_or(LendingError::MathOverflow)?
        .checked_div(collateral_mint_supply as u128)
        .ok_or(LendingError::MathOverflow)?
        as u64;

    msg!("🔍 Collateral underlying: {} assets", underlying_collateral);

    // ========================================================================
    // STEP 5: Get prices from Pyth Oracle
    // ========================================================================
    let price_update_col = &ctx.accounts.price_update_col;
    let price_update_borrow = &ctx.accounts.price_update_borrow;
    
    // Get collateral price using feed ID from market state
    let collateral_feed_id = collateral_market.pyth_feed_id;
    // msg!("Collateral Feed ID: {}", hex::encode(collateral_market.pyth_feed_id));
    let collateral_price_data = price_update_col.get_price_no_older_than(
    &clock, 
    MAXIMUM_AGE, 
    &collateral_feed_id
  )?;

  msg!("collateral_price_data: {:?}", collateral_price_data);
    
    // Get borrow asset price
    let borrow_feed_id = borrow_market.pyth_feed_id;
    let borrow_price_data = price_update_borrow.get_price_no_older_than(
        &clock, 
        MAXIMUM_AGE, 
        &borrow_feed_id
    )?;
    msg!("borrow_price_data: {:?}", borrow_price_data);

    // ========================================================================
    // STEP 6: Calculate values in USD
    // ========================================================================
    let decimals_collateral = ctx.accounts.collateral_mint.decimals;
    let decimals_borrow = ctx.accounts.borrow_mint.decimals;
    
    let collateral_price_normalized = normalize_price(
        collateral_price_data.price,
        collateral_price_data.exponent
    )?;
    msg!("🔍 Collateral Price: ${:?}", collateral_price_normalized);
    
    let borrow_price_normalized = normalize_price(
        borrow_price_data.price,
        borrow_price_data.exponent
    )?;

    msg!("🔍 Borrow Asset Price: ${:?}", borrow_price_normalized);
    
    let collateral_value_usd = (underlying_collateral as u128)
        .checked_mul( collateral_price_data.price as u128)
        .ok_or(LendingError::MathOverflow)?
        .checked_div(10u128.pow(decimals_collateral as u32))
        .ok_or(LendingError::MathOverflow)?;

    msg!("🔍 Collateral Value: ${}", collateral_value_usd);
    
    let borrow_value_usd = (borrow_amount as u128)
        .checked_mul(borrow_price_data.price as u128)
        .ok_or(LendingError::MathOverflow)?
        .checked_div(10u128.pow(decimals_borrow as u32))
        .ok_or(LendingError::MathOverflow)?;

    msg!("🔍 Borrow Value: ${}", borrow_value_usd);

    // ========================================================================
    // STEP 7: Calculate and validate LTV
    // ========================================================================
    let ltv_bps = borrow_value_usd
        .checked_mul(BASIS_POINTS as u128)
        .ok_or(LendingError::MathOverflow)?
        .checked_div(collateral_value_usd)
        .ok_or(LendingError::MathOverflow)?
        as u64;

    
    msg!("🔍 LTV Check: {}bps (max: {}bps)", ltv_bps, borrow_market.max_ltv);
    msg!("   Collateral value: ${}", collateral_value_usd);
    msg!("   Borrow value: ${}", borrow_value_usd);
    
    require!(
        ltv_bps <= borrow_market.max_ltv,
        LendingError::LTVExceeded
    );

    // ========================================================================
    // STEP 8: Calculate borrow fee and debt shares
    // ========================================================================
    let borrow_fee = borrow_amount
        .checked_mul(borrow_market.borrow_fee)
        .ok_or(LendingError::MathOverflow)?
        .checked_div(BASIS_POINTS)
        .ok_or(LendingError::MathOverflow)?;
    
    let net_borrow = borrow_amount
        .checked_sub(borrow_fee)
        .ok_or(LendingError::MathOverflow)?;
    
    // Calculate debt shares (dTokens)
    let total_dtokens = borrow_market.total_borrowed_shares;
    let total_borrows = borrow_market.total_borrows;
    
    let debt_shares = if total_dtokens == 0 || total_borrows == 0 {
        borrow_amount // 1:1 initial
    } else {
        (borrow_amount as u128)
            .checked_mul(total_dtokens as u128)
            .ok_or(LendingError::MathOverflow)?
            .checked_div(total_borrows as u128)
            .ok_or(LendingError::MathOverflow)?
            as u64
    };

    // ========================================================================
    // STEP 9: Lock collateral rTokens
    // ========================================================================
    collateral_position.locked_collateral = collateral_position.locked_collateral
        .checked_add(shares_amount)
        .ok_or(LendingError::MathOverflow)?;
    
    let loan = &mut ctx.accounts.loan;

    // ========================================================================
    // STEP 10: Create/Update loan record
    // ========================================================================
    loan.borrower = ctx.accounts.borrower.key();
    loan.collateral_market = collateral_market.key();
    loan.collateral_amount = shares_amount;
    loan.borrow_market = borrow_market.key();
    loan.borrowed_amount = debt_shares;
    loan.borrowed_underlying = borrow_amount;
    loan.current_market = borrow_market.mint;
    loan.current_position_value = net_borrow;
    loan.l3_integration = Pubkey::default();
    // loan.status = LoanStatus::Active;  
    // loan.current_spent_status = SpentStatus::NotSpent;  
    loan.created_at = clock.unix_timestamp;
    loan.updated_at = clock.unix_timestamp;
    loan.user_position_account = collateral_position.key();

    market_protocol_state.total_loans = market_protocol_state.total_loans
        .checked_add(1)
        .ok_or(LendingError::MathOverflow)?;
    
    // ========================================================================
    // STEP 11: Update market states
    // ========================================================================
    borrow_market.total_borrows = borrow_market.total_borrows
        .checked_add(borrow_amount)
        .ok_or(LendingError::MathOverflow)?;
    borrow_market.total_borrowed_shares = borrow_market.total_borrowed_shares
        .checked_add(debt_shares)
        .ok_or(LendingError::MathOverflow)?;
    
    msg!("   Loan created successfully!");
    msg!("   Loan ID: {}", loan.loan_id);
    msg!("   Collateral: {} rTokens", shares_amount);
    msg!("   Borrowed: {} assets ({} dTokens)", net_borrow, debt_shares);
    msg!("   LTV: {}%", ltv_bps / 100);
    msg!("   Fee: {}", borrow_fee);
    
    Ok(loan.loan_id)
}
