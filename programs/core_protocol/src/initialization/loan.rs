use core::borrow;

use crate::constants::{ANCHOR_DISCRIMINATOR_SIZE, PRECISION};
use crate::errors::LendingError;
use crate::state::{ProtocolState,Loan,Market};
use anchor_lang::prelude::*;


#[derive(Accounts)]
#[instruction(borrow_asset:Pubkey,collateral_asset:Pubkey)]
pub struct InitializeLoan<'info> {
    #[account(mut)]
    pub borrower: Signer<'info>,

    /// Market account (PDA)
    #[account(
        constraint = supply_market.mint == collateral_asset @ LendingError::InvalidMarket,
        seeds = [b"market", collateral_asset.key().as_ref()],
        bump = supply_market.bump,
        
    )]
    pub supply_market: Account<'info, Market>,

    #[account(
        constraint = borrow_market.mint == borrow_asset @ LendingError::InvalidMarket,
        seeds = [b"market", borrow_asset.key().as_ref()],
        bump = borrow_market.bump,
        
    )]
    pub borrow_market: Account<'info, Market>,

    #[account(
        seeds = [b"protocol_state"],
        bump = protocol_state.bump,
    )]
    pub protocol_state: Account<'info, ProtocolState>,


    #[account(
        init,
        payer = borrower,
        space = Loan::LEN,
        seeds = [b"loan",supply_market.key().as_ref(),borrow_market.key().as_ref(),borrower.key().as_ref()],
        bump
    )]
    pub loan: Account<'info, Loan>,

    pub system_program: Program<'info, System>,
}


pub fn handler_initialize_loan(
    ctx: Context<InitializeLoan>,borrow_asset:Pubkey,collateral_asset:Pubkey
) -> Result<()> {
    let loan = &mut ctx.accounts.loan;
    loan.borrower = ctx.accounts.borrower.key();
    loan.loan_id = ctx.accounts.protocol_state.total_loans + 1;
    // Initialize other fields as needed
    Ok(())
} 