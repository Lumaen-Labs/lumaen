use crate::constants::{ANCHOR_DISCRIMINATOR_SIZE, PRECISION};
use crate::errors::LendingError;
use crate::state::{Market, ProtocolState};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

// ============================================================================
// INSTRUCTION 2: Initialize Market (Per asset, by admin)
// ============================================================================
#[derive(Accounts)]
pub struct InitializeMarket<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        seeds = [b"protocol_state"],
        bump = protocol_state.bump,
        constraint = protocol_state.admin == owner.key() @ LendingError::Unauthorized,
    )]
    pub protocol_state: Account<'info, ProtocolState>,

    /// The underlying asset mint (e.g., SOL, USDC, USDT)
    pub underlying_mint: InterfaceAccount<'info, Mint>,

    /// Market account (PDA)
    #[account(
        init,
        payer = owner,
        space = ANCHOR_DISCRIMINATOR_SIZE + Market::INIT_SPACE ,
        seeds = [b"market", underlying_mint.key().as_ref()],
        bump,
    )]
    pub market: Account<'info, Market>,

    /// Supply vault - holds all deposited assets (PDA token account)
    #[account(
        init,
        payer = owner,
        token::mint = underlying_mint,
        token::authority = supply_vault, // Self-authority via PDA
        seeds = [b"supply_vault", market.key().as_ref()],
        bump,
    )]
    pub supply_vault: InterfaceAccount<'info, TokenAccount>,

    // Receipt token (rToken) mint - PDA mint
    // #[account(
    //     init,
    //     payer = owner,
    //     mint::decimals = underlying_mint.decimals,
    //     mint::authority = market, // Market is mint authority
    //     seeds = [b"rtoken_mint", market.key().as_ref()],
    //     bump,
    // )]
    // pub rtoken_mint: InterfaceAccount<'info, Mint>,

    // Debt token (dToken) mint - PDA mint
    // #[account(
    //     init,
    //     payer = owner,
    //     mint::decimals = underlying_mint.decimals,
    //     mint::authority = market, // Market is mint authority
    //     seeds = [b"dtoken_mint", market.key().as_ref()],
    //     bump,
    // )]
    // pub dtoken_mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    // pub rent: Sysvar<'info, Rent>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct MarketConfig {
    // Risk parameters
    pub max_ltv: u64,               // e.g., 7500 = 75%
    pub liquidation_threshold: u64, // e.g., 8000 = 80%
    pub liquidation_penalty: u64,   // e.g., 500 = 5%
    pub reserve_factor: u64,        // e.g., 1000 = 10%

    // Limits
    pub min_deposit_amount: u64,
    pub max_deposit_amount: u64,
    pub min_borrow_amount: u64,
    pub max_borrow_amount: u64,

    // Fees (basis points)
    pub deposit_fee: u64, // e.g., 10 = 0.1%
    pub withdraw_fee: u64,
    pub borrow_fee: u64,
    pub repay_fee: u64,
    // Interest rate model
    // pub base_rate: u64,                  // e.g., 200 = 2%
    // pub optimal_utilization: u64,        // e.g., 8000 = 80%
    // pub slope1: u64,                     // e.g., 400 = 4%
    // pub slope2: u64,                     // e.g., 7500 = 75%
}

pub fn handler_initialize_market(
    ctx: Context<InitializeMarket>,
    config: MarketConfig,
) -> Result<()> {
    let market = &mut ctx.accounts.market;
    let clock = Clock::get()?;

    // Basic info
    market.mint = ctx.accounts.underlying_mint.key();
    market.supply_vault = ctx.accounts.supply_vault.key();
    // market.rtoken_mint = ctx.accounts.rtoken_mint.key();
    // market.dtoken_mint = ctx.accounts.dtoken_mint.key();

    // Financial state (start at zero)
    market.total_deposits = 0;
    market.total_deposited_shares = 0;
    market.total_borrowed_shares = 0;
    market.total_borrows = 0;
    market.total_reserves = 0;

    // Interest tracking
    market.last_update_timestamp = clock.unix_timestamp;
    market.supply_index = PRECISION; // Start at 1.0
    market.borrow_index = PRECISION; // Start at 1.0

    // Risk parameters from config
    market.max_ltv = config.max_ltv;
    market.liquidation_threshold = config.liquidation_threshold;
    market.liquidation_penalty = config.liquidation_penalty;
    market.reserve_factor = config.reserve_factor;

    // Limits from config
    market.min_deposit_amount = config.min_deposit_amount;
    market.max_deposit_amount = config.max_deposit_amount;
    market.min_borrow_amount = config.min_borrow_amount;
    market.max_borrow_amount = config.max_borrow_amount;

    // Daily withdraw limit
    market.last_withdraw_reset_time = clock.unix_timestamp;
    market.deposit_snapshot = 0;

    // Fees from config
    market.deposit_fee = config.deposit_fee;
    market.withdraw_fee = config.withdraw_fee;
    market.borrow_fee = config.borrow_fee;
    market.repay_fee = config.repay_fee;

    // Interest rate model from config
    // market.base_rate = config.base_rate;
    // market.optimal_utilization = config.optimal_utilization;
    // market.slope1 = config.slope1;
    // market.slope2 = config.slope2;

    market.paused = false;
    market.bump = ctx.bumps.market;

    msg!("  Market initialized for mint: {}", market.mint);
    msg!("   Supply Vault: {}", market.supply_vault);
    // msg!("   rToken Mint: {}", market.rtoken_mint);
    // msg!("   dToken Mint: {}", market.dtoken_mint);
    msg!("   Max LTV: {}%", market.max_ltv / 100);
    msg!(
        "   Liquidation Threshold: {}%",
        market.liquidation_threshold / 100
    );

    Ok(())
}
