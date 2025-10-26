// use crate::constants::*;
// use crate::errors::*;
// use crate::instructions::helper::*;
// use crate::state::*;
// use anchor_lang::prelude::*;
// use anchor_spl::associated_token::AssociatedToken;
// use anchor_spl::token_interface::{
//     self, Mint, TokenAccount, TokenInterface, TransferChecked,
// };

// #[derive(Accounts)]
// pub struct Deposit<'info> {
//     // user
//     #[account(mut)]
//     pub signer: Signer<'info>,

//     pub mint: InterfaceAccount<'info, Mint>,

//     // get the associated market config
//     #[account(
//         mut,
//         seeds = [b"market", mint.key().as_ref()],
//         bump = market.bump,
//         constraint = market.paused == false @ LendingError::MarketPaused,
//     )]
//     pub market: Account<'info, Market>,

//     // User's source token account (e.g., user's USDC)
//     #[account(
//         mut,
//         associated_token::mint = market.mint,
//         associated_token::authority = signer,
//         associated_token::token_program = token_program,

//     )]
//     pub user_token_account: InterfaceAccount<'info, TokenAccount>,

//     #[account(
//         mut,
//         seeds = [b"supply_vault", market.key().as_ref()],
//         bump
//     )]
//     pub supply_vault: InterfaceAccount<'info, TokenAccount>,


//     // User's position tracking account
//     #[account(
//         mut,
//         // payer = signer,
//         // space = ANCHOR_DISCRIMINATOR_SIZE + UserPosition::INIT_SPACE,
//         seeds = [b"user_account", signer.key().as_ref(), market.key().as_ref()],
//         bump,
//         constraint = user_account.user == signer.key() @ LendingError::Unauthorized
//     )]
//     pub user_account: Account<'info, UserPosition>,

//     pub token_program: Interface<'info, TokenInterface>,
//     pub associated_token_program: Program<'info, AssociatedToken>,
//     pub system_program: Program<'info, System>,
// }

// pub fn deposit_handler(ctx: Context<Deposit>, amount: u64) -> Result<(u64)> {
//     // init_if_needed the position will automatically create if not exists
//     // same thing can be done for token accounts
//     let user_account = &mut ctx.accounts.user_account;
//     let market = &mut ctx.accounts.market;

//     let clock = Clock::get()?;

//     // ========================================================================
//     // STEP 1: Validate deposit amount
//     // ========================================================================
//     require!(
//         amount >= market.min_deposit_amount,
//         LendingError::DepositTooSmall
//     );
//     require!(
//         amount <= market.max_deposit_amount,
//         LendingError::DepositTooLarge
//     );

//     // ========================================================================
//     // STEP 2: Accrue interest BEFORE any state changes
//     // ========================================================================
//     // TODO: This is where you'd integrate with an interest rate model
//     // For now, using a simple linear model based on utilization
//     accrue_interest(market, clock.unix_timestamp)?;

//     // ========================================================================
//     // STEP 3: Calculate deposit fee
//     // ========================================================================
//     let fee_amount = amount
//         .checked_mul(market.deposit_fee)
//         .ok_or(LendingError::MathOverflow)?
//         .checked_div(BASIS_POINTS)
//         .ok_or(LendingError::MathOverflow)?;

//     let deposit_after_fee = amount
//         .checked_sub(fee_amount)
//         .ok_or(LendingError::MathOverflow)?;

//     // ========================================================================
//     // STEP 4: Calculate exchange rate and rTokens to mint
//     // ========================================================================
//     // Exchange rate = total_assets / total_rtoken_supply
//     // If first deposit, rate = 1:1

//     // let total_rtokens = ctx.accounts.rtoken_mint.supply;
//     let total_rtokens = market.total_deposited_shares;
//     let total_assets = market.total_deposits; // Already includes accrued interest

//     let rtokens_to_mint = if total_rtokens == 0 || total_assets == 0 {
//         deposit_after_fee // 1:1 initial rate
//     } else {
//         // rtokens = deposit * (total_rtokens / total_assets)
//         (deposit_after_fee as u128)
//             .checked_mul(total_rtokens as u128)
//             .ok_or(LendingError::MathOverflow)?
//             .checked_div(total_assets as u128)
//             .ok_or(LendingError::MathOverflow)? as u64
//     };

//     require!(rtokens_to_mint > 0, LendingError::DepositTooSmall);

//     //@ need to check the call
//     // ========================================================================
//     // STEP 5: Transfer tokens from user to supply vault
//     // =======================================================================

//     let transfer_cpi_accounts = TransferChecked {
//         from: ctx.accounts.user_token_account.to_account_info(),
//         mint: ctx.accounts.mint.to_account_info(),
//         to: ctx.accounts.supply_vault.to_account_info(),
//         authority: ctx.accounts.signer.to_account_info(),
//     };

//     let cpi_program = ctx.accounts.token_program.to_account_info();
//     let cpi_ctx = CpiContext::new(cpi_program, transfer_cpi_accounts);
//     let decimals = ctx.accounts.mint.decimals;

//     token_interface::transfer_checked(cpi_ctx, amount, decimals)?;

//     // ========================================================================
//     // STEP 6: Transfer fee to collector
//     // ========================================================================
//     // if fee_amount > 0 {
//     //     let fee_transfer_ctx = CpiContext::new(
//     //         ctx.accounts.token_program.to_account_info(),
//     //         Transfer {
//     //             from: ctx.accounts.supply_vault.to_account_info(),
//     //             to: ctx.accounts.fee_collector_account.to_account_info(),
//     //             authority: ctx.accounts.supply_vault.to_account_info(),
//     //         },
//     //     );
//     //      // Need to sign with supply_vault PDA
//     //      let market_key = market.key();
//     //      let seeds = &[
//     //          b"supply_vault",
//     //          market_key.as_ref(),
//     //          &[ctx.bumps.supply_vault],
//     //      ];
//     //      let signer = &[&seeds[..]];
//     //      token::transfer(fee_transfer_ctx.with_signer(signer), fee_amount)?;
//     // }

//     // ========================================================================
//     // STEP 7: Mint rTokens to user
//     // ========================================================================
//     // let market_key = market.key();
//     // let seeds = &[
//     //     b"market",
//     //     market.mint.as_ref(),
//     //     &[market.bump],
//     // ];
//     // let signer = &[&seeds[..]];

//     // let mint_ctx = CpiContext::new_with_signer(
//     //     ctx.accounts.token_program.to_account_info(),
//     //     MintTo {
//     //         mint: ctx.accounts.rtoken_mint.to_account_info(),
//     //         to: ctx.accounts.user_rtoken_account.to_account_info(),
//     //         authority: market.to_account_info(),
//     //     },
//     //     signer,
//     // );
//     // token::mint_to(mint_ctx, rtokens_to_mint)?;

//     // ========================================================================
//     // STEP 8: Update state
//     // ========================================================================
//     market.total_deposits = market
//         .total_deposits
//         .checked_add(deposit_after_fee)
//         .ok_or(LendingError::MathOverflow)?;
//     market.total_deposited_shares = market
//         .total_deposited_shares
//         .checked_add(rtokens_to_mint)
//         .ok_or(LendingError::MathOverflow)?;
//     // Initialize user position if first time
//     // if user_position.user == Pubkey::default() {
//     //     user_position.user = ctx.accounts.user.key();
//     //     user_position.market = market.key();
//     //     user_position.bump = ctx.bumps.user_position;
//     // }

//     user_account.deposited_shares = user_account
//         .deposited_shares
//         .checked_add(rtokens_to_mint)
//         .ok_or(LendingError::MathOverflow)?;

//     user_account.deposit_index = market.supply_index;

//     msg!(
//         "✅ Deposit successful: {} → {} rTokens",
//         amount,
//         rtokens_to_mint
//     );
//     msg!("   Fee collected: {}", fee_amount);
//     msg!(
//         "   Exchange rate: {} assets per rToken",
//         if total_rtokens > 0 {
//             total_assets / total_rtokens
//         } else {
//             1
//         }
//     );

//     Ok((rtokens_to_mint))
// }




use crate::constants::*;
use crate::errors::*;
use crate::instructions::helper::*;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    self, Mint, TokenAccount, TokenInterface, TransferChecked,
};
#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [b"market", mint.key().as_ref()],
        bump = market.bump,
        constraint = market.paused == false @ LendingError::MarketPaused,
    )]
    pub market: Account<'info, Market>,

    #[account(
        mut,
        associated_token::mint = market.mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program,
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"supply_vault", market.key().as_ref()],
        bump
    )]
    pub supply_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = signer,
        space = ANCHOR_DISCRIMINATOR_SIZE + UserPosition::INIT_SPACE,
        seeds = [b"user_account", signer.key().as_ref(), market.key().as_ref()],
        bump,
    )]
    pub user_account: Account<'info, UserPosition>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn deposit_handler(ctx: Context<Deposit>, amount: u64) -> Result<u64> {
    let user_account = &mut ctx.accounts.user_account;
    let market = &mut ctx.accounts.market;
    let clock = Clock::get()?;

    // Check if existing account belongs to signer (for security on existing accounts)
    if user_account.user != Pubkey::default() {
        require_keys_eq!(user_account.user, ctx.accounts.signer.key(), LendingError::Unauthorized);
    }

    // Validate deposit amount
    require!(amount >= market.min_deposit_amount, LendingError::DepositTooSmall);
    require!(amount <= market.max_deposit_amount, LendingError::DepositTooLarge);

    // Accrue interest
    accrue_interest(market, clock.unix_timestamp)?;


    // Calculate fee and deposit after fee
    let fee_amount = amount
        .checked_mul(market.deposit_fee)
        .ok_or(LendingError::MathOverflow)?
        .checked_div(BASIS_POINTS)
        .ok_or(LendingError::MathOverflow)?;
    let deposit_after_fee = amount
        .checked_sub(fee_amount)
        .ok_or(LendingError::MathOverflow)?;

    // Calculate shares to mint
    let total_shares = market.total_deposited_shares;
    let total_assets = market.total_deposits;
    let shares_to_mint = if total_shares == 0 || total_assets == 0 {
        deposit_after_fee
    } else {
        (deposit_after_fee as u128)
            .checked_mul(total_shares as u128)
            .ok_or(LendingError::MathOverflow)?
            .checked_div(total_assets as u128)
            .ok_or(LendingError::MathOverflow)? as u64
    };
    require!(shares_to_mint > 0, LendingError::DepositTooSmall);

    // Transfer tokens to vault
    let transfer_cpi_accounts = TransferChecked {
        from: ctx.accounts.user_token_account.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.supply_vault.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, transfer_cpi_accounts);
    let decimals = ctx.accounts.mint.decimals;
    token_interface::transfer_checked(cpi_ctx, amount, decimals)?;

    // Handle fee as reserves
    market.total_reserves = market.total_reserves.checked_add(fee_amount).ok_or(LendingError::MathOverflow)?;

    // Update market state
    market.total_deposits = market.total_deposits.checked_add(deposit_after_fee).ok_or(LendingError::MathOverflow)?;
    market.total_deposited_shares = market.total_deposited_shares.checked_add(shares_to_mint).ok_or(LendingError::MathOverflow)?;

    let market_key = market.key();
    // Initialize or update user position
    if user_account.user == Pubkey::default() {
        user_account.user = ctx.accounts.signer.key();
        user_account.market = market_key;
        user_account.bump = ctx.bumps.user_account;
        user_account.deposited_shares = 0; // Explicitly set defaults if needed
        user_account.locked_collateral = 0;
        user_account.borrowed_shares = 0;
        user_account.deposit_index = 0;
        user_account.borrow_index = 0;
    }

    user_account.deposited_shares = user_account.deposited_shares.checked_add(shares_to_mint).ok_or(LendingError::MathOverflow)?;
    user_account.deposit_index = market.supply_index;

    msg!("✅ Deposit successful: {} → {} shares", amount, shares_to_mint);
    msg!("   Fee collected: {}", fee_amount);

    Ok(shares_to_mint)
}