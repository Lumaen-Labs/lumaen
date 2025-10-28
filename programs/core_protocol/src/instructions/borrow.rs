// // // use core::borrow;

// // // use crate::constants::*;
// // // use crate::errors::*;
// // // use crate::initialization::loan;
// // // use crate::instructions::helper::*;
// // // use crate::state::*;
// // // use anchor_lang::prelude::*;
// // // use anchor_spl::associated_token::AssociatedToken;
// // // use anchor_spl::token_interface::{
// // //     self, Mint, TokenAccount, TokenInterface, TransferChecked,
// // // };
// // // use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};
// // // use hex;


// // // #[derive(Accounts)]
// // // pub struct Borrow<'info>{

// // //     // borrower
// // //     #[account(mut)]
// // //     pub borrower: Signer<'info>,

// // //     pub collateral_mint: InterfaceAccount<'info, Mint>,
// // //     pub borrow_mint: InterfaceAccount<'info, Mint>,

// // //     // protocol state to update the total borrows
// // //     #[account(
// // //         seeds = [b"protocol_state"],
// // //         bump = protocol_state.bump,
// // //     )]
// // //     pub protocol_state: Account<'info, ProtocolState>,

// // //     // Collateral market (e.g., USDC) for updating  
// // //     #[account(
// // //         mut,
// // //         seeds = [b"market", collateral_market.mint.as_ref()],
// // //         bump = collateral_market.bump,

// // //     )]
// // //     pub collateral_market: Account<'info, Market>,

// // //     #[account(
// // //         mut,
// // //         seeds = [b"market", borrow_market.mint.as_ref()],
// // //         bump = borrow_market.bump,
// // //         constraint = !borrow_market.paused @ LendingError::MarketPaused,
// // //     )]
// // //     pub borrow_market: Account<'info, Market>,

// // //     #[account(
// // //         mut,
// // //         seeds = [b"user_position", borrower.key().as_ref(), collateral_market.key().as_ref()],
// // //         bump = collateral_position.bump,
// // //     )]
// // //     pub collateral_position: Account<'info, UserPosition>,

// // //      #[account(
// // //         mut,
// // //         seeds = [b"loan",collateral_market.key().as_ref(),borrow_market.key().as_ref(),borrower.key().as_ref()],
// // //         bump = loan.bump,
// // //     )]
// // //     pub loan: Account<'info, Loan>,

// // //     pub token_program: Interface<'info, TokenInterface>,
// // //     pub price_update: Account<'info, PriceUpdateV2>,
// // //     pub system_program: Program<'info, System>,
    
// // // }

// // // pub fn borrow_handler(
// // //     ctx: Context<Borrow>,
// // //     shares_amount:u64,
// // //     borrow_amount: u64,
// // // ) -> Result<(u64)> {
// // //     // Implement the borrow logic here

// // //     // assertions:
// // //     // 1- User has enough collateral deposited 
// // //     // 2 - borrow markets has enough  

// // //        let collateral_market = &mut ctx.accounts.collateral_market;
// // //     let borrow_market = &mut ctx.accounts.borrow_market;
// // //     let collateral_position = &mut ctx.accounts.collateral_position;
// // //     let clock = Clock::get()?;

// // //     // ========================================================================
// // //     // STEP 1: Validate loan amount
// // //     // ========================================================================
// // //     require!(
// // //         borrow_amount >= borrow_market.min_borrow_amount,
// // //         LendingError::BorrowTooSmall
// // //     );
// // //     require!(
// // //         borrow_amount <= borrow_market.max_borrow_amount,
// // //         LendingError::BorrowTooLarge
// // //     );

// // //     // ========================================================================
// // //     // STEP 2: Check user has enough FREE rTokens for collateral
// // //     // ========================================================================
// // //     let free_rtokens = collateral_position.free_rtokens();
// // //     require!(
// // //         shares_amount <= free_rtokens,
// // //         LendingError::InsufficientCollateral
// // //     );

// // //     // ========================================================================
// // //     // STEP 3: Accrue interest on both markets
// // //     // ========================================================================
// // //     accrue_interest(collateral_market, clock.unix_timestamp)?;
// // //     accrue_interest(borrow_market, clock.unix_timestamp)?;

// // //     // ========================================================================
// // //     // STEP 4: Convert rTokens to underlying collateral value
// // //     // ========================================================================
// // //     let collateral_rtokens_supply = collateral_market.total_deposits; 
// // //     let collateral_mint_supply = collateral_market.total_deposited_shares;
    
// // //     let underlying_collateral = (shares_amount as u128)
// // //         .checked_mul(collateral_rtokens_supply as u128)
// // //         .ok_or(LendingError::MathOverflow)?
// // //         .checked_div(collateral_mint_supply as u128)
// // //         .ok_or(LendingError::MathOverflow)?
// // //         as u64;

// // //     // ========================================================================
// // //     // STEP 5: Get prices from Pyth Oracle
// // //     // ========================================================================
// // //     let price_update = &ctx.accounts.price_update;
// // //     let clock = Clock::get()?;
    
// // //     // Get collateral price using feed ID from market state
// // //     let collateral_feed_id = get_feed_id_from_hex(
// // //         &hex::encode(collateral_market.pyth_feed_id)
// // //     )?;
// // //     let collateral_price_data = price_update.get_price_no_older_than(
// // //         &clock, 
// // //         MAXIMUM_AGE, 
// // //         &collateral_feed_id
// // //     )?;
    
// // //     // Get borrow asset price
// // //     let borrow_feed_id = get_feed_id_from_hex(
// // //         &hex::encode(borrow_market.pyth_feed_id)
// // //     )?;
// // //     let borrow_price_data = price_update.get_price_no_older_than(
// // //         &clock, 
// // //         MAXIMUM_AGE, 
// // //         &borrow_feed_id
// // //     )?;

// // //     // ========================================================================
// // //     // STEP 6: Calculate values in USD
// // //     // ========================================================================
// // //     // Pyth prices have variable exponents, so we need to normalize
// // //     let decimals_collateral = ctx.accounts.collateral_mint.decimals;
// // //     let decimals_borrow = ctx.accounts.borrow_mint.decimals;
// // //     let collateral_price_normalized = normalize_price(
// // //         collateral_price_data.price,
// // //         collateral_price_data.exponent
// // //     )?;
    
// // //     let borrow_price_normalized = normalize_price(
// // //         borrow_price_data.price,
// // //         borrow_price_data.exponent
// // //     )?;
    
// // //     let collateral_value_usd = (underlying_collateral as u128)
// // //         .checked_mul(collateral_price_normalized as u128)
// // //         .ok_or(LendingError::MathOverflow)?
// // //         .checked_div(10u128.pow(decimals_collateral as u32))
// // //         .ok_or(LendingError::MathOverflow)?;
    
// // //     let borrow_value_usd = (borrow_amount as u128)
// // //         .checked_mul(borrow_price_normalized as u128)
// // //         .ok_or(LendingError::MathOverflow)?
// // //         .checked_div(10u128.pow(decimals_borrow as u32))
// // //         .ok_or(LendingError::MathOverflow)?;

// // //     // ========================================================================
// // //     // STEP 7: Calculate and validate LTV
// // //     // ========================================================================
// // //     let ltv_bps = borrow_value_usd
// // //         .checked_mul(BASIS_POINTS as u128)
// // //         .ok_or(LendingError::MathOverflow)?
// // //         .checked_div(collateral_value_usd)
// // //         .ok_or(LendingError::MathOverflow)?
// // //         as u64;
    
// // //     msg!("🔍 LTV Check: {}bps (max: {}bps)", ltv_bps, borrow_market.max_ltv);
// // //     msg!("   Collateral value: ${}", collateral_value_usd);
// // //     msg!("   Borrow value: ${}", borrow_value_usd);
    
// // //     require!(
// // //         ltv_bps <= borrow_market.max_ltv,
// // //         LendingError::LTVExceeded
// // //     );

// // //     // ========================================================================
// // //     // STEP 8: Check liquidity in borrow vault
// // //     // ========================================================================
// // //     // let vault_balance = ctx.accounts.borrow_supply_vault.amount;
// // //     // require!(
// // //     //     borrow_amount <= vault_balance,
// // //     //     LendingError::InsufficientLiquidity
// // //     // );

// // //         // ========================================================================
// // //     // STEP 7: Calculate borrow fee and debt shares
// // //     // ========================================================================
// // //     let borrow_fee = borrow_amount
// // //         .checked_mul(borrow_market.borrow_fee)
// // //         .ok_or(LendingError::MathOverflow)?
// // //         .checked_div(BASIS_POINTS)
// // //         .ok_or(LendingError::MathOverflow)?;
    
// // //     let net_borrow = borrow_amount
// // //         .checked_sub(borrow_fee)
// // //         .ok_or(LendingError::MathOverflow)?;
    
// // //     // Calculate debt shares (dTokens)
// // //     let total_dtokens = borrow_market.total_borrowed_shares;
// // //     let total_borrows = borrow_market.total_borrows;
    
// // //     let debt_shares = if total_dtokens == 0 || total_borrows == 0 {
// // //         borrow_amount // 1:1 initial
// // //     } else {
// // //         (borrow_amount as u128)
// // //             .checked_mul(total_dtokens as u128)
// // //             .ok_or(LendingError::MathOverflow)?
// // //             .checked_div(total_borrows as u128)
// // //             .ok_or(LendingError::MathOverflow)?
// // //             as u64
// // //     };


// // //     // ========================================================================
// // //     // STEP 8: Lock collateral rTokens
// // //     // ========================================================================
// // //     collateral_position.locked_collateral = collateral_position.locked_collateral
// // //         .checked_add(shares_amount)
// // //         .ok_or(LendingError::MathOverflow)?;
    
// // //     let loan = &mut ctx.accounts.loan;

// // //     // ========================================================================
// // //     // STEP 9: Create loan record
// // //     // ========================================================================
// // //     loan.collateral_market = collateral_market.key();
// // //     loan.collateral_amount = shares_amount;
// // //     loan.borrow_market = borrow_market.key();
// // //     loan.borrowed_amount = debt_shares;
// // //     loan.borrowed_underlying = borrow_amount;
// // //     loan.current_market = borrow_market.mint; // Currently with user
// // //     loan.current_amount = net_borrow;
// // //     loan.l3_integration = Pubkey::default(); // Not spent yet
// // //     // loan.status = LoanStatus::Active;
// // //     loan.status_u8 = 0;
// // //     loan.current_spent_u8 = 0;

// // //     loan.created_at = clock.unix_timestamp;
// // //     loan.updated_at = clock.unix_timestamp;
// // //     loan.user_position_account = collateral_position.key();
    
// // //     // ========================================================================
// // //     // STEP 12: Update market states
// // //     // ========================================================================
// // //     borrow_market.total_borrows = borrow_market.total_borrows
// // //         .checked_add(borrow_amount)
// // //         .ok_or(LendingError::MathOverflow)?;
// // //     borrow_market.total_borrowed_shares = borrow_market.total_borrowed_shares
// // //         .checked_add(debt_shares)
// // //         .ok_or(LendingError::MathOverflow)?;
    
// // //     // Initialize borrow position if needed
// // //     // borrow_position.borrowed_shares = borrow_position.borrowed_shares
// // //     //     .checked_add(debt_shares)
// // //     //     .ok_or(LendingError::MathOverflow)?;
// // //     // borrow_position.borrow_index = borrow_market.borrow_index;
    
// // //     // msg!("✅ Loan created successfully!");
// // //     // msg!("   Loan ID: {}", loan_id);
// // //     // msg!("   Collateral: {} rTokens", collateral_rtoken_amount);
// // //     // msg!("   Borrowed: {} assets ({} dTokens)", net_borrow, debt_shares);
// // //     // msg!("   LTV: {}%", ltv_bps / 100);
// // //     // msg!("   Fee: {}", borrow_fee);
    
// // //     Ok((loan.loan_id))
// // // }



use core::borrow;

use crate::constants::*;
use crate::errors::*;
use crate::initialization::loan;
use crate::instructions::helper::*;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    self, Mint, TokenAccount, TokenInterface, TransferChecked,
};
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};
use hex;


// #[derive(Accounts)]
// pub struct Borrow<'info>{

//     // borrower
//     #[account(mut)]
//     pub borrower: Signer<'info>,

//     pub collateral_mint: InterfaceAccount<'info, Mint>,
//     pub borrow_mint: InterfaceAccount<'info, Mint>,

//     // protocol state to update the total borrows
//     #[account(
//         seeds = [b"protocol_state"],
//         bump,
//     )]
//     pub protocol_state: Account<'info, ProtocolState>, 

//     // Collateral market (e.g., USDC) for updating  
//     #[account(
//         mut,
//         seeds = [b"market", collateral_mint.key().as_ref()],
//         bump = collateral_market.bump,
//     )]
//     pub collateral_market: Account<'info, Market>, 

//     #[account(
//         mut,
//         seeds = [b"market", borrow_mint.key().as_ref()],
//         bump = borrow_market.bump,
//         constraint = !borrow_market.paused @ LendingError::MarketPaused,
//     )]
//     pub borrow_market: Account<'info, Market>, 

//     #[account(
//         mut,
//         seeds = [b"user_account", borrower.key().as_ref(), collateral_market.key().as_ref()],
//         bump = collateral_position.bump,
//     )]
//     pub collateral_position: Account<'info, UserPosition>, 

//     #[account(
//         mut,
//         seeds = [b"loan", collateral_market.key().as_ref(), borrow_market.key().as_ref(), borrower.key().as_ref()],
//         bump = loan.bump,
//     )]
//     pub loan: Account<'info, Loan>,  

//     pub token_program: Interface<'info, TokenInterface>,
//     pub price_update: Account<'info, PriceUpdateV2>,
//     pub system_program: Program<'info, System>,
// }


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
    pub protocol_state: Account<'info, ProtocolState>,

    // Collateral market (e.g., USDC) for updating  
    #[account(
        mut,
        seeds = [b"market", collateral_mint.key().as_ref()],
        bump
    )]
    pub collateral_market: Account<'info, Market>,

    #[account(
        mut,
        seeds = [b"market", borrow_mint.key().as_ref()],
        bump ,
        constraint = !borrow_market.paused @ LendingError::MarketPaused,
    )]
    pub borrow_market: Account<'info, Market>,

    #[account(
        mut,
        seeds = [b"user_account", borrower.key().as_ref(), collateral_market.key().as_ref()],
        bump,
    )]
    pub collateral_position: Account<'info, UserPosition>,
    
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
    pub loan: Account<'info, Loan>,

    pub token_program: Interface<'info, TokenInterface>,
    pub price_update: Account<'info, PriceUpdateV2>,
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
    let collateral_rtokens_supply = collateral_market.total_deposits; 
    let collateral_mint_supply = collateral_market.total_deposited_shares;
    
    let underlying_collateral = (shares_amount as u128)
        .checked_mul(collateral_rtokens_supply as u128)
        .ok_or(LendingError::MathOverflow)?
        .checked_div(collateral_mint_supply as u128)
        .ok_or(LendingError::MathOverflow)?
        as u64;

    // ========================================================================
    // STEP 5: Get prices from Pyth Oracle
    // ========================================================================
    let price_update = &ctx.accounts.price_update;
    
    // Get collateral price using feed ID from market state
    let collateral_feed_id = get_feed_id_from_hex(
        &hex::encode(collateral_market.pyth_feed_id)
    )?;
    let collateral_price_data = price_update.get_price_no_older_than(
        &clock, 
        MAXIMUM_AGE, 
        &collateral_feed_id
    )?;
    
    // Get borrow asset price
    let borrow_feed_id = get_feed_id_from_hex(
        &hex::encode(borrow_market.pyth_feed_id)
    )?;
    let borrow_price_data = price_update.get_price_no_older_than(
        &clock, 
        MAXIMUM_AGE, 
        &borrow_feed_id
    )?;

    // ========================================================================
    // STEP 6: Calculate values in USD
    // ========================================================================
    let decimals_collateral = ctx.accounts.collateral_mint.decimals;
    let decimals_borrow = ctx.accounts.borrow_mint.decimals;
    
    let collateral_price_normalized = normalize_price(
        collateral_price_data.price,
        collateral_price_data.exponent
    )?;
    
    let borrow_price_normalized = normalize_price(
        borrow_price_data.price,
        borrow_price_data.exponent
    )?;
    
    let collateral_value_usd = (underlying_collateral as u128)
        .checked_mul(collateral_price_normalized as u128)
        .ok_or(LendingError::MathOverflow)?
        .checked_div(10u128.pow(decimals_collateral as u32))
        .ok_or(LendingError::MathOverflow)?;
    
    let borrow_value_usd = (borrow_amount as u128)
        .checked_mul(borrow_price_normalized as u128)
        .ok_or(LendingError::MathOverflow)?
        .checked_div(10u128.pow(decimals_borrow as u32))
        .ok_or(LendingError::MathOverflow)?;

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




// // use core::borrow;
// use core::borrow;

// use crate::constants::*;
// use crate::errors::*;
// use crate::initialization::loan;
// use crate::instructions::helper::*;
// use crate::state::*;
// use anchor_lang::prelude::*;
// use anchor_spl::associated_token::AssociatedToken;
// use anchor_spl::token_interface::{
//     self, Mint, TokenAccount, TokenInterface, TransferChecked,
// };
// use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};
// use hex;


// #[derive(Accounts)]
// pub struct Borrow<'info>{

//     // borrower
//     #[account(mut)]
//     pub borrower: Signer<'info>,

//     pub collateral_mint: InterfaceAccount<'info, Mint>,
//     pub borrow_mint: InterfaceAccount<'info, Mint>,

//     // protocol state to update the total borrows
//     #[account(
//         mut,
//         seeds = [b"protocol_state"],
//         bump
//     )]
//     pub protocol_state: Account<'info, ProtocolState>,

//     // Collateral market (e.g., USDC) for updating  
//     #[account(
//         mut,
//         seeds = [b"market", collateral_mint.key().as_ref()],
//         bump
//     )]
//     pub collateral_market: Account<'info, Market>,

//     #[account(
//         mut,
//         seeds = [b"market", borrow_mint.key().as_ref()],
//         bump ,
//         constraint = !borrow_market.paused @ LendingError::MarketPaused,
//     )]
//     pub borrow_market: Account<'info, Market>,

//     #[account(
//         mut,
//         seeds = [b"user_account", borrower.key().as_ref(), collateral_market.key().as_ref()],
//         bump,
//     )]
//     pub collateral_position: Account<'info, UserPosition>,
    
//     // #[account(
//     //     mut,
//     //     seeds = [b"supply_vault", market.key().as_ref()],
//     //     bump
//     // )]
//     // pub borrow_vault: InterfaceAccount<'info, TokenAccount>,

//     #[account(
//         init_if_needed,
//         payer = borrower,
//         space = ANCHOR_DISCRIMINATOR_SIZE + Loan::LEN,
//         seeds = [b"loan", collateral_market.key().as_ref(), borrow_market.key().as_ref(), borrower.key().as_ref()],
//         bump,
//     )]
//     pub loan: Account<'info, Loan>,

//     pub token_program: Interface<'info, TokenInterface>,
//     pub price_update: Account<'info, PriceUpdateV2>,
//     pub system_program: Program<'info, System>,
// }

// pub fn borrow_handler(
//     ctx: Context<Borrow>,
//     shares_amount: u64,
//     borrow_amount: u64,
// ) -> Result<u64> {
//     let protocol_state = &mut ctx.accounts.protocol_state;
//     let collateral_market = &mut ctx.accounts.collateral_market;
//     let borrow_market = &mut ctx.accounts.borrow_market;
//     let collateral_position = &mut ctx.accounts.collateral_position;
//     let loan = &mut ctx.accounts.loan;
//     let clock = Clock::get()?;

//     // ========================================================================
//     // STEP 1: Validate deposit amount
//     // ========================================================================
//     require!(
//         borrow_amount >= borrow_market.min_borrow_amount,
//         LendingError::BorrowTooSmall
//     );
//     require!(
//         borrow_amount <= borrow_market.max_borrow_amount,
//         LendingError::BorrowTooLarge
//     );

//     // ========================================================================
//     // STEP 2: Check user has enough FREE rTokens for collateral
//     // ========================================================================
//     let free_rtokens = collateral_position.free_rtokens();
//     require!(
//         shares_amount <= free_rtokens,
//         LendingError::InsufficientCollateral
//     );

//     // ========================================================================
//     // STEP 3: Accrue interest on both markets
//     // ========================================================================
//     accrue_interest(collateral_market, clock.unix_timestamp)?;
//     accrue_interest(borrow_market, clock.unix_timestamp)?;

//     // ========================================================================
//     // STEP 4: Calculate deposit fee
//     // ========================================================================
//     let fee_amount = borrow_amount
//         .checked_mul(borrow_market.borrow_fee)
//         .ok_or(LendingError::MathOverflow)?
//         .checked_div(BASIS_POINTS)
//         .ok_or(LendingError::MathOverflow)?;

//     let borrow_after_fee = borrow_amount
//         .checked_sub(fee_amount)
//         .ok_or(LendingError::MathOverflow)?;

//     // ========================================================================
//     // STEP 5: Calculate exchange rate and rTokens to mint
//     // ========================================================================
//     // Exchange rate = total_assets / total_rtoken_supply
//     // If first deposit, rate = 1:1

//     // let total_rtokens = ctx.accounts.rtoken_mint.supply;
//     let total_dtokens = borrow_market.total_borrowed_shares;
//     let total_assets = borrow_market.total_borrows; // Already includes accrued interest

//     let dtokens_to_mint = if total_dtokens == 0 || total_assets == 0 {
//         borrow_after_fee // 1:1 initial rate
//     } else {
//         // rtokens = deposit * (total_rtokens / total_assets)
//         (borrow_after_fee as u128)
//             .checked_mul(total_dtokens as u128)
//             .ok_or(LendingError::MathOverflow)?
//             .checked_div(total_assets as u128)
//             .ok_or(LendingError::MathOverflow)? as u64
//     };

//     require!(dtokens_to_mint > 0, LendingError::BorrowTooSmall);

//     //@ need to check the call
//     // ========================================================================
//     // STEP 6: Transfer tokens from user to supply vault
//     // =======================================================================

//     // let transfer_cpi_accounts = TransferChecked {
//     //     from: ctx.accounts.to_account_info(),
//     //     mint: ctx.accounts.borrow_mint.to_account_info(),
//     //     to: ctx.accounts.borrower_token_account.to_account_info(),
//     //     authority: ctx.accounts.borrow_vault.to_account_info(),
//     // };

//     // let cpi_program = ctx.accounts.token_program.to_account_info();
//     // let cpi_ctx = CpiContext::new(cpi_program, transfer_cpi_accounts);
//     // let decimals = ctx.accounts.borrow_mint.decimals;

//     // token_interface::transfer_checked(cpi_ctx, borrow_amount, decimals)?;

//     // ========================================================================
//     // STEP 7: Handle fee as reserves (since no transfer, add to reserves)
//     // ========================================================================
//     borrow_market.total_reserves = borrow_market
//         .total_reserves
//         .checked_add(fee_amount)
//         .ok_or(LendingError::MathOverflow)?;

//     // ========================================================================
//     // STEP 8: Update state
//     // ========================================================================
//     borrow_market.total_borrows = borrow_market
//         .total_borrows
//         .checked_add(borrow_after_fee)
//         .ok_or(LendingError::MathOverflow)?;
//     borrow_market.total_borrowed_shares = borrow_market
//         .total_borrowed_shares
//         .checked_add(dtokens_to_mint)
//         .ok_or(LendingError::MathOverflow)?;
//     // Initialize user position if first time
//     if collateral_position.user == Pubkey::default() {
//         collateral_position.user = ctx.accounts.borrower.key();
//         collateral_position.market = collateral_market.key();
//         collateral_position.bump = ctx.bumps.collateral_position;
//     }

//     collateral_position.locked_collateral = collateral_position
//         .locked_collateral
//         .checked_add(shares_amount)
//         .ok_or(LendingError::MathOverflow)?;

//     // Initialize loan if first time
//     if loan.borrower == Pubkey::default() {
//         loan.borrower = ctx.accounts.borrower.key();
//         loan.loan_id = protocol_state.total_loans;
//         loan.collateral_market = collateral_market.key();
//         loan.collateral_amount = shares_amount;
//         loan.borrow_market = borrow_market.key();
//         loan.borrowed_amount = dtokens_to_mint;
//         loan.borrowed_underlying = borrow_amount;
//         loan.user_position_account = collateral_position.key();
//         loan.current_market = borrow_market.mint;
//         loan.current_amount = borrow_after_fee;
//         loan.l3_integration = Pubkey::default();
//         loan.l3_shares_received = 0;
//         loan.current_spent_u8 = 0;
//         loan.status_u8 = 0;
//         loan.created_at = clock.unix_timestamp;
//         loan.updated_at = clock.unix_timestamp;
//         loan.bump = ctx.bumps.loan;
//         protocol_state.total_loans = protocol_state.total_loans.checked_add(1).ok_or(LendingError::MathOverflow)?;
//     } else {
//         // Update existing loan if needed, but assuming one loan per pair
//         loan.collateral_amount += shares_amount;
//         loan.borrowed_amount += dtokens_to_mint;
//         loan.borrowed_underlying += borrow_amount;
//         loan.current_amount += borrow_after_fee;
//         loan.updated_at = clock.unix_timestamp;
//     }

//     msg!(
//         "✅ Borrow successful: {} → {} dTokens",
//         borrow_amount,
//         dtokens_to_mint
//     );
//     msg!("   Fee collected: {}", fee_amount);
//     msg!(
//         "   Exchange rate: {} assets per dToken",
//         if total_dtokens > 0 {
//             total_assets / total_dtokens
//         } else {
//             1
//         }
//     );

//     Ok((dtokens_to_mint))
// }