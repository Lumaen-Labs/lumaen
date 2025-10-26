// use crate::constants::{ANCHOR_DISCRIMINATOR_SIZE, PRECISION};
// use crate::errors::LendingError;
// use crate::state::{Market, ProtocolState,MarketConfig};
// use anchor_lang::prelude::*;
// use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

// // ============================================================================
// // INSTRUCTION 2: Initialize Market (Per asset, by admin)
// // ============================================================================
// #[derive(Accounts)]
// pub struct InitializeMarket<'info> {
//     #[account(mut)]
//     pub owner: Signer<'info>,

//     #[account(
//         seeds = [b"protocol_state"],
//         bump = protocol_state.bump,
//         constraint = protocol_state.admin == owner.key() @ LendingError::Unauthorized,
//     )]
//     pub protocol_state: Account<'info, ProtocolState>,

//     /// The underlying asset mint (e.g., SOL, USDC, USDT)
//     pub mint: InterfaceAccount<'info, Mint>,

//     /// Market account (PDA)
//     #[account(
//         init,
//         payer = owner,
//         space = ANCHOR_DISCRIMINATOR_SIZE + Market::INIT_SPACE ,
//         seeds = [b"market", mint.key().as_ref()],
//         bump,
//     )]
//     pub market: Account<'info, Market>,

//     /// Supply vault - holds all deposited assets (PDA token account)
//     #[account(
//         init,
//         payer = owner,
//         token::mint = mint,
//         token::authority = supply_vault, // Self-authority via PDA
//         seeds = [b"supply_vault", market.key().as_ref()],
//         bump,
//     )]
//     pub supply_vault: InterfaceAccount<'info, TokenAccount>,
//     pub token_program: Interface<'info, TokenInterface>,
//     pub system_program: Program<'info, System>,
// }

// pub fn handler_initialize_market(
//     ctx: Context<InitializeMarket>,
//     config: MarketConfig,
// ) -> Result<()> {
//     let market = &mut ctx.accounts.market;
//     let clock = Clock::get()?;

//     // Basic info
//     market.mint = ctx.accounts.mint.key();
//     market.supply_vault = ctx.accounts.supply_vault.key();

//     // Financial state (start at zero)
//     market.total_deposits = 0;
//     market.total_deposited_shares = 0;
//     market.total_borrowed_shares = 0;
//     market.total_borrows = 0;
//     market.total_reserves = 0;

//     // Interest tracking
//     market.last_update_timestamp = clock.unix_timestamp;
//     market.supply_index = PRECISION; // Start at 1.0
//     market.borrow_index = PRECISION; // Start at 1.0

//     // Risk parameters from config
//     market.max_ltv = config.max_ltv;
//     market.liquidation_threshold = config.liquidation_threshold;
//     market.liquidation_penalty = config.liquidation_penalty;
//     market.reserve_factor = config.reserve_factor;

//     // Limits from config
//     market.min_deposit_amount = config.min_deposit_amount;
//     market.max_deposit_amount = config.max_deposit_amount;
//     market.min_borrow_amount = config.min_borrow_amount;
//     market.max_borrow_amount = config.max_borrow_amount;

//     // Daily withdraw limit
//     market.last_withdraw_reset_time = clock.unix_timestamp;
//     market.deposit_snapshot = 0;

//     // Fees from config
//     market.deposit_fee = config.deposit_fee;
//     market.withdraw_fee = config.withdraw_fee;
//     market.borrow_fee = config.borrow_fee;
//     market.repay_fee = config.repay_fee;

//     // Interest rate model from config
//     // market.base_rate = config.base_rate;
//     // market.optimal_utilization = config.optimal_utilization;
//     // market.slope1 = config.slope1;
//     // market.slope2 = config.slope2;

//     market.paused = false;
//     market.bump = ctx.bumps.market;

//     msg!("  Market initialized for mint: {}", market.mint);
//     msg!("   Supply Vault: {}", market.supply_vault);
//     // msg!("   rToken Mint: {}", market.rtoken_mint);
//     // msg!("   dToken Mint: {}", market.dtoken_mint);
//     msg!("   Max LTV: {}%", market.max_ltv / 100);
//     msg!(
//         "   Liquidation Threshold: {}%",
//         market.liquidation_threshold / 100
//     );

//     Ok(())
// }



use crate::constants::{ANCHOR_DISCRIMINATOR_SIZE, PRECISION};
use crate::errors::LendingError;
use crate::state::{Market, ProtocolState,MarketConfig};
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
        mut,
        seeds = [b"protocol_state"],
        bump = protocol_state.bump,
        constraint = protocol_state.admin == owner.key() @ LendingError::Unauthorized,
    )]
    pub protocol_state: Account<'info, ProtocolState>,

    /// The underlying asset mint (e.g., SOL, USDC, USDT)
    pub mint: InterfaceAccount<'info, Mint>,

    /// Market account (PDA)
    #[account(
        init,
        payer = owner,
        space = ANCHOR_DISCRIMINATOR_SIZE + Market::INIT_SPACE ,
        seeds = [b"market", mint.key().as_ref()],
        bump,
    )]
    pub market: Account<'info, Market>,

    /// Supply vault - holds all deposited assets (PDA token account)
    #[account(
        init,
        payer = owner,
        token::mint = mint,
        token::authority = supply_vault, // Self-authority via PDA
        seeds = [b"supply_vault", market.key().as_ref()],
        bump,
    )]
    pub supply_vault: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn handler_initialize_market(
    ctx: Context<InitializeMarket>,
    config: MarketConfig,
) -> Result<()> {
    let market = &mut ctx.accounts.market;
    let clock = Clock::get()?;

    // Basic info
    market.mint = ctx.accounts.mint.key();
    market.supply_vault = ctx.accounts.supply_vault.key();

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