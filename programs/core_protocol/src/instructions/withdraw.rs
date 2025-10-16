use crate::constants::*;
use crate::errors::*;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    self, Burn, Mint, TokenAccount, TokenInterface, TransferChecked,
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

    // #[account(
    //     seeds = [b"protocol_state"],
    //     bump = protocol_state.bump,
    // )]
    // pub protocol_state: Account<'info, ProtocolState>,
    #[account(
        mut,
        seeds = [b"market", market.mint.as_ref()],
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

    // #[account(mut)]
    // pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    // #[account(
    //     mut,
    //     address = market.rtoken_mint,
    // )]
    // pub rtoken_mint: Account<'info, Mint>,

    // #[account(
    //     mut,
    //     constraint = user_rtoken_account.mint == market.rtoken_mint,
    //     constraint = user_rtoken_account.owner == user.key(),
    // )]
    // pub user_rtoken_account: Account<'info, TokenAccount>,

    // #[account(
    //     mut,
    //     seeds = [b"user_position", signer.key().as_ref(), market.key().as_ref()],
    //     bump = user_position.bump,
    // )]
    // pub user_position: Account<'info, UserPosition>,

    //  #[account(
    //     mut,
    //     // payer = signer,
    //     // space = ANCHOR_DISCRIMINATOR_SIZE + UserPosition::INIT_SPACE,
    //     // seeds = [b"user_position", user.key().as_ref(), market.key().as_ref()],
    //     // bump,
    //     constraint = user_position.user == signer.key() @ LendingError::Unauthorized
    // )]
    // pub user_position: Account<'info, UserPosition>,
    #[account(
        mut, 
        seeds =[b"user_account",signer.key().as_ref(),market.mint.as_ref()],
        bump,
    )]
    pub user_position: Account<'info, UserPosition>,

    #[account( 
        init_if_needed, 
        payer = signer,
        associated_token::mint = mint, 
        associated_token::authority = signer,
        associated_token::token_program = token_program,
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    // #[account(
    //     mut,
    //     constraint = fee_collector_account.mint == market.mint,
    // )]
    // pub fee_collector_account: Account<'info, TokenAccount>,

    // pub token_program: Program<'info, Token>,
}

pub fn withdraw_handler(ctx: Context<Withdraw>, rtoken_amount: u64) -> Result<()> {
    // let market = &mut ctx.accounts.market;
    let user_position = &mut ctx.accounts.user_position;
    let clock = Clock::get()?;

    // ========================================================================
    // STEP 1: Check user has enough FREE (unlocked) rTokens
    // ========================================================================
    let free_rtokens = user_position.free_rtokens();
    require!(
        rtoken_amount <= free_rtokens,
        LendingError::InsufficientFreeRTokens
    );

    // ========================================================================
    // STEP 2: Accrue interest
    // ========================================================================
    accrue_interest(&mut ctx.accounts.market, clock.unix_timestamp)?;

    // ========================================================================
    // STEP 3: Calculate underlying assets to withdraw
    // ========================================================================
    let total_rtokens = ctx.accounts.market.total_deposited_shares;
    let total_assets = ctx.accounts.market.total_deposits;

    let underlying_amount = (rtoken_amount as u128)
        .checked_mul(total_assets as u128)
        .ok_or(LendingError::MathOverflow)?
        .checked_div(total_rtokens as u128)
        .ok_or(LendingError::MathOverflow)? as u64;

    // ========================================================================
    // STEP 4: Check liquidity in vault
    // ========================================================================
    let vault_balance = ctx.accounts.supply_vault.amount;
    require!(
        underlying_amount <= vault_balance,
        LendingError::InsufficientLiquidity
    );

    // // ========================================================================
    // // STEP 5: Daily withdraw limit check (20% of reserves per 24h)
    // // ========================================================================
    // let current_time = clock.unix_timestamp;

    // // Reset snapshot if 24 hours passed
    // if current_time >= market.last_withdraw_reset_time + 86400 {
    //     market.last_withdraw_reset_time = current_time;
    //     market.deposit_snapshot = total_assets;
    // }

    // // Calculate minimum allowed reserves after withdrawal
    // let min_reserves = market.deposit_snapshot
    //     .checked_mul(BASIS_POINTS - DAILY_WITHDRAW_LIMIT_BPS)
    //     .ok_or(LendingError::MathOverflow)?
    //     .checked_div(BASIS_POINTS)
    //     .ok_or(LendingError::MathOverflow)?;

    // let post_withdraw_reserves = total_assets
    //     .checked_sub(underlying_amount)
    //     .ok_or(LendingError::MathOverflow)?;

    // require!(
    //     post_withdraw_reserves >= min_reserves,
    //     LendingError::WithdrawLimitExceeded
    // );

    // ========================================================================
    // STEP 6: Calculate withdrawal fee
    // ========================================================================
    let fee_amount = underlying_amount
        .checked_mul(ctx.accounts.market.withdraw_fee)
        .ok_or(LendingError::MathOverflow)?
        .checked_div(BASIS_POINTS)
        .ok_or(LendingError::MathOverflow)?;

    let withdraw_after_fee = underlying_amount
        .checked_sub(fee_amount)
        .ok_or(LendingError::MathOverflow)?;

    // ========================================================================
    // STEP 7: Burn user's rTokens
    // ========================================================================
    // let burn_ctx = CpiContext::new(
    //     ctx.accounts.token_program.to_account_info(),
    //     Burn {
    //         mint: ctx.accounts.rtoken_mint.to_account_info(),
    //         from: ctx.accounts.user_rtoken_account.to_account_info(),
    //         authority: ctx.accounts.user.to_account_info(),
    //     },
    // );
    // token::burn(burn_ctx, rtoken_amount)?;

    // ========================================================================
    // STEP 8: Transfer underlying assets to user
    // ========================================================================
    // let market_key = market.key();
    // let seeds = &[
    //     b"supply_vault",
    //     market_key.as_ref(),
    //     &[ctx.bumps.supply_vault],
    // ];
    // let signer = &[&seeds[..]];

    // let transfer_ctx = CpiContext::new_with_signer(
    //     ctx.accounts.token_program.to_account_info(),
    //     Transfer {
    //         from: ctx.accounts.supply_vault.to_account_info(),
    //         to: ctx.accounts.user_token_account.to_account_info(),
    //         authority: ctx.accounts.supply_vault.to_account_info(),
    //     },
    //     signer,
    // );
    // token::transfer(transfer_ctx, withdraw_after_fee)?;


     let transfer_cpi_accounts = TransferChecked {
        from: ctx.accounts.supply_vault.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.supply_vault.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let mint_key = ctx.accounts.mint.key();
    let market_key = ctx.accounts.market.key();
    let signer_seeds: &[&[&[u8]]] = &[
        &[
            b"supply_vault",
            market_key.as_ref(),
            &[ctx.bumps.supply_vault],
        ],
    ];
    let cpi_ctx = CpiContext::new(cpi_program, transfer_cpi_accounts).with_signer(signer_seeds);

    let decimals = ctx.accounts.mint.decimals;

    token_interface::transfer_checked(cpi_ctx, withdraw_after_fee, decimals)?;


    // ========================================================================
    // STEP 9: Transfer fee
    // ========================================================================
    // if fee_amount > 0 {
    //     let fee_ctx = CpiContext::new_with_signer(
    //         ctx.accounts.token_program.to_account_info(),
    //         Transfer {
    //             from: ctx.accounts.supply_vault.to_account_info(),
    //             to: ctx.accounts.fee_collector_account.to_account_info(),
    //             authority: ctx.accounts.supply_vault.to_account_info(),
    //         },
    //         signer,
    //     );
    //     token::transfer(fee_ctx, fee_amount)?;
    // }

    // ========================================================================
    // STEP 10: Update state
    // ========================================================================
    let market = &mut ctx.accounts.market;
    market.total_deposits = market
        .total_deposits
        .checked_sub(underlying_amount)
        .ok_or(LendingError::MathOverflow)?;
    market.total_deposited_shares = market
        .total_deposited_shares
        .checked_sub(rtoken_amount)
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

    Ok(())
}