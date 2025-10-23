use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_spl::token_interface::{
    Mint, TokenAccount, TokenInterface, TransferChecked
};
use crate::state::{Loan, Market, UserPosition};
use crate::errors::LendingError;
use crate::constants::BASIS_POINTS;

// Solend program IDs (devnet)
pub const SOLEND_PROGRAM_ID: &str = "ALend7Ketfx5bxh6ghsCDXAoDrhvEmsXT3cynB6aPLgx";

// Solend instruction discriminators
pub const DEPOSIT_RESERVE_LIQUIDITY: u8 = 14;
pub const REDEEM_RESERVE_COLLATERAL: u8 = 15;

// ============================================================================
// INVEST IN SOLEND
// ============================================================================
#[derive(Accounts)]
pub struct InvestInSolend<'info> {
    #[account(mut)]
    pub borrower: Signer<'info>,

    // Loan that tracks the borrowed funds
    #[account(
        mut,
        seeds = [
            b"loan",
            loan.collateral_market.as_ref(),
            loan.borrow_market.as_ref(),
            borrower.key().as_ref()
        ],
        bump = loan.bump,
        constraint = loan.borrower == borrower.key() @ LendingError::InvalidLoan,
        constraint = loan.status_u8 == 0 @ LendingError::InvalidLoan, // Must be Active
        constraint = loan.current_spent_u8 == 0 @ LendingError::InvalidLoan, // Must be NotSpent
    )]
    pub loan: Box<Account<'info, Loan>>,

    // Borrow market to track where funds came from
    #[account(
        mut,
        address = loan.borrow_market,
    )]
    pub borrow_market: Box<Account<'info, Market>>,

    // Our protocol's vault holding the borrowed funds
    #[account(
        mut,
        seeds = [b"supply_vault", borrow_market.key().as_ref()],
        bump,
    )]
    pub protocol_vault: InterfaceAccount<'info, TokenAccount>,

    // Solend's reserve liquidity supply (where we deposit)
    /// CHECK: Validated by Solend program
    pub solend_liquidity_supply: AccountInfo<'info>,

    // Solend's reserve account for this token
    /// CHECK: Validated by Solend program
    pub solend_reserve: AccountInfo<'info>,

    // Solend's collateral token mint (cToken)
    /// CHECK: Validated by Solend program
    pub solend_ctoken_mint: AccountInfo<'info>,

    // Protocol's cToken account (PDA to hold Solend receipt tokens)
    #[account(
        init_if_needed,
        payer = borrower,
        token::mint = solend_ctoken_mint,
        token::authority = loan_ctoken_vault, // Self-authority
        seeds = [b"loan_ctoken_vault", loan.key().as_ref()],
        bump,
    )]
    pub loan_ctoken_vault: InterfaceAccount<'info, TokenAccount>,

    // Solend's reserve collateral supply (cToken destination)
    /// CHECK: Validated by Solend program
    pub solend_collateral_supply: AccountInfo<'info>,

    // Solend program
    /// CHECK: Program ID validation
    #[account(
        constraint = solend_program.key().to_string() == SOLEND_PROGRAM_ID
    )]
    pub solend_program: AccountInfo<'info>,

    // Solend's lending market account
    /// CHECK: Validated by Solend program
    pub solend_market: AccountInfo<'info>,

    // Solend's lending market authority
    /// CHECK: Validated by Solend program
    pub solend_market_authority: AccountInfo<'info>,

    pub underlying_mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

pub fn handler_invest_in_solend(
    ctx: Context<InvestInSolend>
) -> Result<()> {
    let loan = &mut ctx.accounts.loan;
    let borrow_market = &ctx.accounts.borrow_market;
    let clock = Clock::get()?;

    // ========================================================================
    // STEP 1: Validate amount
    // ========================================================================
    // require!(
    //     amount <= loan.current_amount,
    //     LendingError::InsufficientLiquidity
    // );   
    let amount = loan.current_amount;
    require!(amount > 0, LendingError::InvalidLoan);

    // ========================================================================
    // STEP 2: Build Solend deposit instruction
    // ========================================================================
    let mut deposit_data = vec![DEPOSIT_RESERVE_LIQUIDITY];
    deposit_data.extend_from_slice(&amount.to_le_bytes());

    let deposit_accounts = vec![
        // Source liquidity token account
        AccountMeta::new(ctx.accounts.protocol_vault.key(), false),
        // Destination collateral token account (our cToken vault)
        AccountMeta::new(ctx.accounts.loan_ctoken_vault.key(), false),
        // Reserve account
        AccountMeta::new(ctx.accounts.solend_reserve.key(), false),
        // Reserve liquidity supply
        AccountMeta::new(ctx.accounts.solend_liquidity_supply.key(), false),
        // Reserve collateral mint
        AccountMeta::new(ctx.accounts.solend_ctoken_mint.key(), false),
        // Lending market account
        AccountMeta::new(ctx.accounts.solend_market.key(), false),
        // Lending market authority
        AccountMeta::new_readonly(ctx.accounts.solend_market_authority.key(), false),
        // User transfer authority (our vault PDA)
        AccountMeta::new_readonly(ctx.accounts.protocol_vault.key(), true),
        // Clock sysvar
        AccountMeta::new_readonly(ctx.accounts.clock.key(), false),
        // Token program
        AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),
    ];

    let deposit_instruction = Instruction {
        program_id: ctx.accounts.solend_program.key(),
        accounts: deposit_accounts,
        data: deposit_data,
    };

    // ========================================================================
    // STEP 3: Execute CPI to Solend with PDA signer
    // ========================================================================
    let borrow_market_key = borrow_market.key();
    let vault_seeds: &[&[&[u8]]] = &[&[
        b"supply_vault",
        borrow_market_key.as_ref(),
        &[ctx.bumps.protocol_vault],
    ]];

    anchor_lang::solana_program::program::invoke_signed(
        &deposit_instruction,
        &[
            ctx.accounts.protocol_vault.to_account_info(),
            ctx.accounts.loan_ctoken_vault.to_account_info(),
            ctx.accounts.solend_reserve.to_account_info(),
            ctx.accounts.solend_liquidity_supply.to_account_info(),
            ctx.accounts.solend_ctoken_mint.to_account_info(),
            ctx.accounts.solend_market.to_account_info(),
            ctx.accounts.solend_market_authority.to_account_info(),
            ctx.accounts.protocol_vault.to_account_info(),
            ctx.accounts.clock.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
        ],
        vault_seeds,
    )?;

    // ========================================================================
    // STEP 4: Get cToken balance and update loan state
    // ========================================================================
    ctx.accounts.loan_ctoken_vault.reload()?;
    let ctokens_received = ctx.accounts.loan_ctoken_vault.amount;

    loan.current_spent_u8 = 1; // Status: InSolend
    loan.l3_integration = ctx.accounts.solend_program.key();
    loan.current_amount = loan.current_amount
        .checked_sub(amount)
        .ok_or(LendingError::MathOverflow)?;
    loan.l3_shares_received = ctokens_received;
    loan.updated_at = clock.unix_timestamp;

    msg!("✅ Successfully invested in Solend:");
    msg!("   Amount invested: {}", amount);
    msg!("   cTokens received: {}", ctokens_received);
    msg!("   Remaining loan balance: {}", loan.current_amount);

    Ok(())
}

// ============================================================================
// WITHDRAW FROM SOLEND
// ============================================================================
#[derive(Accounts)]
pub struct WithdrawFromSolend<'info> {
    #[account(mut)]
    pub borrower: Signer<'info>,

    // Loan that tracks the position
    #[account(
        mut,
        seeds = [
            b"loan",
            loan.collateral_market.as_ref(),
            loan.borrow_market.as_ref(),
            borrower.key().as_ref()
        ],
        bump = loan.bump,
        constraint = loan.borrower == borrower.key() @ LendingError::InvalidLoan,
        constraint = loan.current_spent_u8 == 1 @ LendingError::InvalidLoan, // Must be InSolend
    )]
    pub loan: Box<Account<'info, Loan>>,

    // Borrow market
    #[account(
        mut,
        address = loan.borrow_market,
    )]
    pub borrow_market: Box<Account<'info, Market>>,

    // Protocol's vault to receive withdrawn funds
    #[account(
        mut,
        seeds = [b"supply_vault", borrow_market.key().as_ref()],
        bump,
    )]
    pub protocol_vault: InterfaceAccount<'info, TokenAccount>,

    // Protocol's cToken account
    #[account(
        mut,
        seeds = [b"loan_ctoken_vault", loan.key().as_ref()],
        bump,
    )]
    pub loan_ctoken_vault: InterfaceAccount<'info, TokenAccount>,

    // Solend accounts
    /// CHECK: Validated by Solend program
    pub solend_reserve: AccountInfo<'info>,
    
    /// CHECK: Validated by Solend program
    pub solend_liquidity_supply: AccountInfo<'info>,
    
    /// CHECK: Validated by Solend program
    pub solend_collateral_supply: AccountInfo<'info>,
    
    /// CHECK: Validated by Solend program
    pub solend_ctoken_mint: AccountInfo<'info>,
    
    /// CHECK: Validated by Solend program
    pub solend_market: AccountInfo<'info>,
    
    /// CHECK: Validated by Solend program
    pub solend_market_authority: AccountInfo<'info>,

    /// CHECK: Program ID validation
    #[account(
        constraint = solend_program.key().to_string() == SOLEND_PROGRAM_ID
    )]
    pub solend_program: AccountInfo<'info>,

    pub underlying_mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

pub fn handler_withdraw_from_solend(
    ctx: Context<WithdrawFromSolend>,
) -> Result<()> {
    let loan = &mut ctx.accounts.loan;
    let clock = Clock::get()?;

    // Check it's actually in Solend
    require!(
        loan.l3_integration == ctx.accounts.solend_program.key(),
        LendingError::InvalidLoan
    );

    // ========================================================================
    // STEP 1: Get all cTokens to redeem
    // ========================================================================
    let ctoken_amount = loan.l3_shares_received;
    require!(ctoken_amount > 0, LendingError::InvalidLoan);

    // ========================================================================
    // STEP 2: Build Solend redeem instruction
    // ========================================================================
    let mut redeem_data = vec![REDEEM_RESERVE_COLLATERAL];
    redeem_data.extend_from_slice(&ctoken_amount.to_le_bytes());

    let redeem_accounts = vec![
        // Source collateral token account (our cToken vault)
        AccountMeta::new(ctx.accounts.loan_ctoken_vault.key(), false),
        // Destination liquidity token account (our protocol vault)
        AccountMeta::new(ctx.accounts.protocol_vault.key(), false),
        // Reserve account
        AccountMeta::new(ctx.accounts.solend_reserve.key(), false),
        // Reserve collateral mint
        AccountMeta::new(ctx.accounts.solend_ctoken_mint.key(), false),
        // Reserve liquidity supply
        AccountMeta::new(ctx.accounts.solend_liquidity_supply.key(), false),
        // Lending market account
        AccountMeta::new(ctx.accounts.solend_market.key(), false),
        // Lending market authority
        AccountMeta::new_readonly(ctx.accounts.solend_market_authority.key(), false),
        // User transfer authority (our cToken vault PDA)
        AccountMeta::new_readonly(ctx.accounts.loan_ctoken_vault.key(), true),
        // Clock sysvar
        AccountMeta::new_readonly(ctx.accounts.clock.key(), false),
        // Token program
        AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),
    ];

    let redeem_instruction = Instruction {
        program_id: ctx.accounts.solend_program.key(),
        accounts: redeem_accounts,
        data: redeem_data,
    };

    // ========================================================================
    // STEP 3: Execute CPI to Solend with PDA signer
    // ========================================================================
    let loan_key = loan.key();
    let ctoken_vault_seeds: &[&[&[u8]]] = &[&[
        b"loan_ctoken_vault",
        loan_key.as_ref(),
        &[ctx.bumps.loan_ctoken_vault],
    ]];

    // Get vault balance before redeem
    let vault_balance_before = ctx.accounts.protocol_vault.amount;

    anchor_lang::solana_program::program::invoke_signed(
        &redeem_instruction,
        &[
            ctx.accounts.loan_ctoken_vault.to_account_info(),
            ctx.accounts.protocol_vault.to_account_info(),
            ctx.accounts.solend_reserve.to_account_info(),
            ctx.accounts.solend_ctoken_mint.to_account_info(),
            ctx.accounts.solend_liquidity_supply.to_account_info(),
            ctx.accounts.solend_market.to_account_info(),
            ctx.accounts.solend_market_authority.to_account_info(),
            ctx.accounts.loan_ctoken_vault.to_account_info(),
            ctx.accounts.clock.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
        ],
        ctoken_vault_seeds,
    )?;

    // ========================================================================
    // STEP 4: Calculate received amount and update loan state
    // ========================================================================
    ctx.accounts.protocol_vault.reload()?;
    let vault_balance_after = ctx.accounts.protocol_vault.amount;
    let underlying_received = vault_balance_after
        .checked_sub(vault_balance_before)
        .ok_or(LendingError::MathOverflow)?;

    // Add withdrawn amount back to current_amount
    loan.current_amount = loan.current_amount
        .checked_add(underlying_received)
        .ok_or(LendingError::MathOverflow)?;
    
    // Reset spent status
    loan.current_spent_u8 = 0; // NotSpent
    loan.l3_integration = Pubkey::default();
    loan.l3_shares_received = 0;
    loan.updated_at = clock.unix_timestamp;

    // Calculate profit (interest earned on Solend)
    let original_amount = loan.borrowed_underlying;
    let interest_earned = if underlying_received > original_amount {
        underlying_received - original_amount
    } else {
        0
    };

    msg!("✅ Successfully withdrawn from Solend:");
    msg!("   cTokens redeemed: {}", ctoken_amount);
    msg!("   Amount received: {}", underlying_received);
    msg!("   Interest earned: {}", interest_earned);
    msg!("   Current loan balance: {}", loan.current_amount);

    // If interest earned exceeds borrow interest, it helps pay down the loan
    // This will be handled during repayment

    Ok(())
}