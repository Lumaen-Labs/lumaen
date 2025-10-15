use anchor_lang::prelude::*;

declare_id!("3AsHpu3rrzQjx1gTAWUKyiqaFj6HdSUFovLEhXpP2Ufv");

pub mod constants;
pub mod errors;
pub mod events;
pub mod initialization;
pub mod instructions;
pub mod state;

use initialization::*;
use instructions::*;

#[program]
pub mod router {
    use super::*;

    // return type should be shares
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<u64> {
        instructions::deposit_handler(ctx, amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, shares: u64) -> Result<u64> {
        instruction::withdraw_handler(ctx, shares)
    }

    pub fn initialize_protocol(ctx: Context<InitializeProtocol>) -> Result<()> {
        initialization::handler_initialize_protocol(ctx)
    }
    pub fn initialize_market(ctx: Context<InitializeMarket>, config: MarketConfig) -> Result<()> {
        initialization::handler_initialize_market(ctx, config)
    }

    pub fn initialize_user_position(
        ctx: Context<InitializeUserRTokenAccount>,
        market_mint: Pubkey,
    ) -> Result<()> {
        initialization::handler_initialize_user_rtoken_account(ctx)
    }
}
