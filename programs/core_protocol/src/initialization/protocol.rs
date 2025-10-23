// ============================================================================
// INITIALIZATION INSTRUCTIONS - Missing from Previous Code
// ============================================================================

use crate::constants::ANCHOR_DISCRIMINATOR_SIZE;
use crate::state::ProtocolState;
use anchor_lang::prelude::*;
// ============================================================================
// INSTRUCTION 1: Initialize Protocol (One-time, by admin)
// ============================================================================
#[derive(Accounts)]
pub struct InitializeProtocol<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    /// CHECK: Fee collector can be any account
    pub fee_collector: AccountInfo<'info>,

    #[account(
        init,
        payer = admin,
        space = ANCHOR_DISCRIMINATOR_SIZE + ProtocolState::INIT_SPACE,
        seeds = [b"protocol_state"],
        bump,
    )]
    pub protocol_state: Account<'info, ProtocolState>,

    pub system_program: Program<'info, System>,
}

pub fn handler_initialize_protocol(ctx: Context<InitializeProtocol>) -> Result<()> {
    let protocol_state = &mut ctx.accounts.protocol_state;

    protocol_state.admin = ctx.accounts.admin.key();
    protocol_state.fee_collector = ctx.accounts.fee_collector.key();
    protocol_state.protocol_paused = false;
    protocol_state.total_markets = 0;
    protocol_state.total_loans = 0;
    protocol_state.bump = ctx.bumps.protocol_state;


    msg!("  Protocol initialized!");
    msg!("   Admin: {}", protocol_state.admin);
    msg!("   Fee Collector: {}", protocol_state.fee_collector);

    Ok(())
}

// Global protocol configuration (Single instance)
// #[account]
// #[derive(InitSpace)]
// pub struct ProtocolState {
//     pub admin: Pubkey,
//     pub fee_collector: Pubkey,
//     pub protocol_paused: bool,
//     pub total_markets: u64,
//     pub total_loans: u64,
//     pub bump: u8,
// }





