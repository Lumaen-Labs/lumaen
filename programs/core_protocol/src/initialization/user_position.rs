use crate::constants::ANCHOR_DISCRIMINATOR_SIZE;
use crate::state::UserPosition;
use anchor_lang::prelude::*;

// ============================================================================
// INSTRUCTION 3: Create User Token Accounts (Separate for flexibility)
// ============================================================================
// CORRECTION: Users only need the token accounts for actions they take:
// - Depositors only need rToken account
// - Borrowers only need dToken account
// - Users doing both will need both, but created separately

// Create rToken account (for depositors/suppliers)
#[derive(Accounts)]
#[instruction(market_mint:Pubkey)]
pub struct InitializeUserPosition<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    // #[account(
    //     seeds = [b"market", market.mint.as_ref()],
    //     bump = market.bump,
    // )]
    // pub market: Account<'info, Market>,
    #[account(
        init,
        payer = signer,
        space = ANCHOR_DISCRIMINATOR_SIZE + UserPosition::INIT_SPACE,
        seeds =[b"user_account",signer.key().as_ref(),market_mint.as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserPosition>,

    /// The rToken mint account (PDA owned by market)
    // #[account(
    //     seeds = [b"rtoken_mint", market.key().as_ref()],
    //     bump,
    // )]
    // pub rtoken_mint: InterfaceAccount<'info, Mint>,

    /// User's rToken account (receipt tokens for deposits)
    // #[account(
    //     init,
    //     payer = user,
    //     token::mint = rtoken_mint,
    //     token::authority = user,
    // )]
    // pub user_rtoken_account: InterfaceAccount<'info, TokenAccount>,

    // pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>, // pub rent: Sysvar<'info, Rent>,
}

pub fn handler_initialize_user_position(
    ctx: Context<InitializeUserPosition>,
    market_mint: Pubkey,
) -> Result<()> {
    let user_account = &mut ctx.accounts.user_account;
    user_account.user = ctx.accounts.signer.key();
    user_account.market = market_mint;
    user_account.deposited_shares = 0;
    user_account.locked_collateral = 0;
    user_account.borrowed_shares = 0;
    user_account.deposit_index = 0;
    user_account.borrow_index = 0;
    user_account.bump = ctx.bumps.user_account;

    // msg!(" User rToken account created!");
    // msg!("   rToken account: {}", ctx.accounts.user_rtoken_account.key());
    // msg!("   User can now deposit to receive rTokens");

    Ok(())
}
// pub user: Pubkey,
// pub market: Pubkey,             // underlying market

// // Deposit tracking
// pub deposited_shares: u64,           // rToken shares owned
// pub locked_collateral: u64,          // rToken shares locked as collateral

// // Borrow tracking
// pub borrowed_shares: u64,            // dToken shares (debt)

// // Interest tracking (for accurate calculations)
// pub deposit_index: u128,             // Last supply index when user interacted
// pub borrow_index: u128,              // Last borrow index when user interacted

// pub bump: u8,
