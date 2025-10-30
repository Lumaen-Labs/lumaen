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