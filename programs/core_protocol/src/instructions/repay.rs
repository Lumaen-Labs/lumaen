
use anchor_lang::prelude::*;
use crate::state::{Loan, ProtocolState, UserPosition, Market};
use crate::errors::*;  
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    self, Mint, TokenAccount, TokenInterface, TransferChecked,
};
use crate::instructions::helper::*;
use crate::constants::BASIS_POINTS;

#[derive(Accounts)]
pub struct Repay<'info>{
    
    // who has taken the loan
    #[account(mut)]
    pub borrower: Signer<'info>,

    pub mint: InterfaceAccount<'info, Mint>,

    // Loan information 
    #[account(
        mut,
        seeds = [b"loan", collateral_market.key().as_ref(), borrow_market.key().as_ref(), borrower.key().as_ref()],
        bump,
        constraint = loan.borrower == borrower.key() @ LendingError::InvalidLoan
    )]
    pub loan: Account<'info, Loan>,

    // collateral market to reduce lets see for now 
    #[account(
        mut,
        address = loan.collateral_market,
    )]
    pub collateral_market: Account<'info, Market>,  

    // Borrow market
    #[account(
        mut,
        address = loan.borrow_market,
    )]
    pub borrow_market: Account<'info, Market>,

    #[account(
        mut,
        seeds = [b"user_account", borrower.key().as_ref(), collateral_market.key().as_ref()],
        bump 
    )]
    pub user_position: Account<'info, UserPosition>,

    // repay amount asset
    #[account(
        mut,
        associated_token::mint = borrow_market.mint,
        associated_token::authority = borrower,
        associated_token::token_program = token_program,
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"supply_vault", borrow_market.key().as_ref()],
        bump
    )]
    pub supply_vault: InterfaceAccount<'info, TokenAccount>,
    
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}


pub fn handler_repay(
    ctx: Context<Repay>,
    repay_amount: u64,
) -> Result<()> {
    let borrow_market = &mut ctx.accounts.borrow_market;
    let loan = &mut ctx.accounts.loan;
    let user_position = &mut ctx.accounts.user_position;

    let clock = Clock::get()?;

    // STEP 1: Accrue interest to get current debt
    accrue_interest(borrow_market, clock.unix_timestamp)?;

    let total_dtokens = borrow_market.total_borrowed_shares;
    let total_borrows = borrow_market.total_borrows;

    let debt_amount = if total_dtokens == 0 {
        0
    } else {
        (loan.borrowed_amount as u128)
            .checked_mul(total_borrows as u128)
            .ok_or(LendingError::MathOverflow)?
            .checked_div(total_dtokens as u128)
            .ok_or(LendingError::MathOverflow)?
        as u64
    };
    
    msg!("💰 Debt calculation:");
    msg!("   Original borrow limit: {}", loan.borrowed_underlying);
    msg!("   Debt shares (dTokens): {}", loan.borrowed_amount);
    msg!("   Current debt with interest: {}", debt_amount);

    // STEP 2: Calculate repay fee
    let repay_fee = debt_amount
        .checked_mul(borrow_market.repay_fee)
        .ok_or(LendingError::MathOverflow)?
        .checked_div(BASIS_POINTS)
        .ok_or(LendingError::MathOverflow)?;
    
    let total_debt = debt_amount
        .checked_add(repay_fee)
        .ok_or(LendingError::MathOverflow)?;

    require!(
        repay_amount >= total_debt,
        LendingError::RepayAmountTooSmall
    );

    // STEP 3: Transfer repayment from user to vault (user pays debt)
    let transfer_cpi_accounts = TransferChecked {
        from: ctx.accounts.user_token_account.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.supply_vault.to_account_info(),
        authority: ctx.accounts.borrower.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, transfer_cpi_accounts);
    let decimals = ctx.accounts.mint.decimals;

    token_interface::transfer_checked(cpi_ctx, total_debt, decimals)?;

    // STEP 4: Calculate net profit (position value - total debt)
    // Assume loan.current_position_value is updated via oracle/strategy
    let net_profit = loan.current_position_value.saturating_sub(total_debt);

    // STEP 5: If profit > 0, transfer back to borrower
    if net_profit > 0 {
        /// @notice 
        // Transfer from vault to user
        // let profit_transfer_accounts = TransferChecked {
        //     from: ctx.accounts.supply_vault.to_account_info(),
        //     mint: ctx.accounts.mint.to_account_info(),
        //     to: ctx.accounts.user_token_account.to_account_info(),
        //     authority: ctx.accounts.supply_vault.to_account_info(),  // PDA signer
        // };
        // let market_key = borrow_market.key();
        // let seeds = &[b"supply_vault".as_ref(), market_key.as_ref(), &[ctx.bumps.supply_vault]];
        // let signer_seeds = &[&seeds[..]];
        // let profit_cpi_ctx = CpiContext::new_with_signer(cpi_program.clone(), profit_transfer_accounts, signer_seeds);
        // token_interface::transfer_checked(profit_cpi_ctx, net_profit, decimals)?;

        msg!("   Profit returned to borrower: {}", net_profit);
    } else if loan.current_position_value < total_debt {
        // Protocol absorbs loss (undercollateralized) - log or handle
        msg!("   Loss absorbed by protocol: {}", total_debt - loan.current_position_value);
    }

    // STEP 6: Unlock collateral
    user_position.locked_collateral = user_position.locked_collateral
        .checked_sub(loan.collateral_amount)
        .ok_or(LendingError::MathOverflow)?;

    // STEP 7: Update market and position states
    borrow_market.total_borrows = borrow_market.total_borrows
        .checked_sub(debt_amount)
        .ok_or(LendingError::MathOverflow)?;
    borrow_market.total_borrowed_shares = borrow_market.total_borrowed_shares
        .checked_sub(loan.borrowed_amount)
        .ok_or(LendingError::MathOverflow)?;
    borrow_market.total_reserves = borrow_market.total_reserves
        .checked_add(repay_fee)
        .ok_or(LendingError::MathOverflow)?;

    // STEP 8: Update loan status and close
    loan.status_u8 = 1;  // Repaid
    loan.updated_at = clock.unix_timestamp;
    
    msg!("✅ Loan repaid successfully!");
    msg!("   Principal repaid: {}", debt_amount);
    msg!("   Interest paid: {}", debt_amount.saturating_sub(loan.borrowed_underlying));
    msg!("   Fee: {}", repay_fee);
    msg!("   Total paid: {}", total_debt);
    msg!("   Collateral unlocked: {} rTokens", loan.collateral_amount);

    // Close loan PDA (via close constraint)
    // If user_position now zero, add close constraint as in withdraw

    Ok(())
}