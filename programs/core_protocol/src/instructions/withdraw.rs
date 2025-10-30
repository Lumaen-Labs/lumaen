use crate::constants::*;
use crate::errors::*;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    self, Mint, TokenAccount, TokenInterface, TransferChecked,
};
use crate::instructions::helper::*;

// ============================================================================
// INSTRUCTION 2: WITHDRAW
// ============================================================================
#[derive(Accounts)]
pub struct Withdraw<'info> {
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
        seeds = [b"supply_vault", market.key().as_ref()],
        bump,
    )]
    pub supply_vault: InterfaceAccount<'info, TokenAccount>,

    #[account( 
        init_if_needed, 
        payer = signer,
        associated_token::mint = mint, 
        associated_token::authority = signer,
        associated_token::token_program = token_program,
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds =[b"user_account",signer.key().as_ref(),market.key().as_ref()],
        bump
    )]
    pub user_position: Account<'info, UserPosition>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn withdraw_handler(ctx: Context<Withdraw>, rtoken_amount: u64) -> Result<()> {
    let market = &mut ctx.accounts.market;
    let user_position = &mut ctx.accounts.user_position;
    let clock = Clock::get()?;

    // STEP 1: Check user has enough FREE (unlocked) rTokens
    let free_rtokens = user_position.free_rtokens();
    require!(
        rtoken_amount <= free_rtokens,
        LendingError::InsufficientFreeRTokens
    );

    // STEP 2: Accrue interest
    accrue_interest(market, clock.unix_timestamp)?;

    // STEP 3: Calculate underlying assets to withdraw
    let total_rtokens = market.total_deposited_shares;
    let total_assets = market.total_deposits;

    let underlying_amount = (rtoken_amount as u128)
        .checked_mul(total_assets as u128)
        .ok_or(LendingError::MathOverflow)?
        .checked_div(total_rtokens as u128)
        .ok_or(LendingError::MathOverflow)? as u64;

    // STEP 4: Check liquidity in vault
    let vault_balance = ctx.accounts.supply_vault.amount;
    require!(
        underlying_amount <= vault_balance,
        LendingError::InsufficientLiquidity
    );

    // STEP 5: Calculate withdrawal fee
    let fee_amount = underlying_amount
        .checked_mul(market.withdraw_fee)
        .ok_or(LendingError::MathOverflow)?
        .checked_div(BASIS_POINTS)
        .ok_or(LendingError::MathOverflow)?;

    let withdraw_after_fee = underlying_amount
        .checked_sub(fee_amount)
        .ok_or(LendingError::MathOverflow)?;

    // STEP 6: Transfer underlying assets to user
    let market_key = market.key();
    let seeds = &[
        b"supply_vault",
        market_key.as_ref(),
        &[ctx.bumps.supply_vault],
    ];
    let signer = &[&seeds[..]];

    let transfer_cpi_accounts = TransferChecked {
        from: ctx.accounts.supply_vault.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.supply_vault.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, transfer_cpi_accounts, signer);
    let decimals = ctx.accounts.mint.decimals;

    token_interface::transfer_checked(cpi_ctx, withdraw_after_fee, decimals)?;

    // STEP 7: Update state
    market.total_deposits = market
        .total_deposits
        .checked_sub(underlying_amount)
        .ok_or(LendingError::MathOverflow)?;
    market.total_deposited_shares = market
        .total_deposited_shares
        .checked_sub(rtoken_amount)
        .ok_or(LendingError::MathOverflow)?;
    market.total_reserves = market
        .total_reserves
        .checked_add(fee_amount)
        .ok_or(LendingError::MathOverflow)?;

    user_position.deposited_shares = user_position
        .deposited_shares
        .checked_sub(rtoken_amount)
        .ok_or(LendingError::MathOverflow)?;

    msg!(
        "✅ Withdraw successful: {} rTokens → {} assets",
        rtoken_amount,
        withdraw_after_fee
    );
    msg!("   Fee: {}", fee_amount);

    // STEP 8: Close position if zero balances (handled via close constraint if all zero after sub)
    Ok(())
}