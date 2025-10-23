use anchor_lang::prelude::*;
use initialization::*;
use instructions::*;
use state::MarketConfig;

mod constants;
mod errors;
mod events;
mod initialization;
mod instructions;
mod state;


declare_id!("7igW79sx9a5aR6XJJve8Uj8QJNrLg1K4A8RqiLZKK9fL");


#[program]
pub mod core_router {
    use super::*;

    // Deposit tokens and receive shares in return
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<u64> {
        deposit_handler(ctx, amount)
    }

    // Withdraw tokens by burning shares
    pub fn withdraw(ctx: Context<Withdraw>, shares: u64) -> Result<()> {
        withdraw_handler(ctx, shares)
    }

    // Initialize the protocol state
    pub fn initialize_protocol(ctx: Context<InitializeProtocol>) -> Result<()> {
        handler_initialize_protocol(ctx)
    }

    // Initialize a new market
    pub fn initialize_market(ctx: Context<InitializeMarket>, config: MarketConfig) -> Result<()> {
        handler_initialize_market(ctx, config)
    }

    // Initialize user position account
    pub fn initialize_user_position(
        ctx: Context<InitializeUserPosition>
    ) -> Result<()> {
        handler_initialize_user_position(ctx)
    }

    pub fn initialize_loan(ctx: Context<InitializeLoan>, borrow_asset: Pubkey, collateral_asset: Pubkey) -> Result<()> {
        handler_initialize_loan(ctx, borrow_asset, collateral_asset)
    }
    pub fn borrow(ctx: Context<Borrow>, shares_amount: u64, borrow_amount: u64) -> Result<(u64)> {
        borrow_handler(ctx, shares_amount, borrow_amount)
    }
    pub fn repay(ctx: Context<Repay>, repay_amount: u64) -> Result<()> {
        handler_repay(ctx, repay_amount)
    }

    pub fn invest_in_solend(
        ctx: Context<InvestInSolend>,
    ) -> Result<()> {
        handler_invest_in_solend(ctx)
    }
    pub fn withdraw_from_solend(
        ctx: Context<WithdrawFromSolend>,
    ) -> Result<()> {
        handler_withdraw_from_solend(ctx)
    }

}
