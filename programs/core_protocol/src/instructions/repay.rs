// // use core::borrow;

// // use anchor_lang::prelude::*;
// // use crate::state::{Loan, ProtocolState,UserPosition,Market};
// // use crate::errors::*;  
// // use anchor_spl::associated_token::AssociatedToken;
// // use anchor_spl::token_interface::{
// //     self, Mint, TokenAccount, TokenInterface, TransferChecked,
// // };
// // use crate::instructions::helper::*;
// // use crate::constants::BASIS_POINTS;

// // #[derive(Accounts)]
// // pub struct Repay<'info>{
    
// //     // who has taken the loan
// //     #[account(mut)]
// //     pub borrower: Signer<'info>,

// //     pub mint: InterfaceAccount<'info, Mint>,

// //     // Loan information 
// //     #[account(
// //         mut,
// //         seeds = [b"loan", borrower.key().as_ref(), &loan.loan_id.to_le_bytes()],
// //         bump = loan.bump,
// //         constraint = loan.borrower == borrower.key() @ LendingError::InvalidLoan,
// //         constraint = loan.status_u8 == 0 @ LendingError::InvalidLoan,
// //     )]
// //     pub loan: Account<'info, Loan>,

// //     // Protocol state
// //     #[account(
// //         seeds = [b"protocol_state"],
// //         bump = protocol_state.bump,
// //     )]
// //     pub protocol_state: Account<'info, ProtocolState>,

// //     // collateral market to reduce lets see for now 
// //     #[account(
// //         mut,
// //         address = loan.collateral_market,
// //     )]
// //     pub collateral_market: Account<'info, Market>,

// //     // Borrow market
// //     #[account(
// //         mut,
// //         address = loan.borrow_market,
// //     )]
// //     pub borrow_market: Account<'info, Market>,

// //     #[account(
// //         mut,
// //         seeds = [b"user_position", borrower.key().as_ref(), collateral_market.key().as_ref()],
// //         bump = user_position.bump,
// //     )]
// //     pub user_position: Account<'info, UserPosition>,

// //     // repay amount asset
// //     #[account(
// //         mut,
// //         associated_token::mint = borrow_market.mint,
// //         associated_token::authority = borrower,
// //         associated_token::token_program = token_program,

// //     )]
// //     pub user_token_account: InterfaceAccount<'info, TokenAccount>,

// //     #[account(
// //         mut,
// //         seeds = [b"supply_vault", borrow_market.key().as_ref()],
// //         bump,
// //     )]
// //     pub supply_vault: InterfaceAccount<'info, TokenAccount>,
    
// //     pub token_program: Interface<'info, TokenInterface>,
// //     pub associated_token_program: Program<'info, AssociatedToken>,
// //     pub system_program: Program<'info, System>,
    
// // }


// // pub fn handler_repay(
// //     ctx: Context<Repay>,
// //     repay_amount: u64,
// // ) -> Result<()> {
// //     // Logic for repaying the loan goes here

// //     let collateral_market = &mut ctx.accounts.collateral_market;
// //     let borrow_market = &mut ctx.accounts.borrow_market;
// //     let loan = &mut ctx.accounts.loan;
// //     let user_position = &mut ctx.accounts.user_position;

// //     let clock = Clock::get()?;

// //     // ========================================================================
// //     // STEP 1: Accrue interest to get current debt
// //     // ========================================================================
// //     accrue_interest(borrow_market, clock.unix_timestamp)?;

// //     let total_dtokens = borrow_market.total_borrowed_shares;
// //     let total_borrows = borrow_market.total_borrows;


// //     let debt_amount = (loan.borrowed_amount as u128)
// //         .checked_mul(total_borrows as u128)
// //         .ok_or(LendingError::MathOverflow)?
// //         .checked_div(total_dtokens as u128)
// //         .ok_or(LendingError::MathOverflow)?
// //         as u64;
    
// //     msg!("  Debt calculation:");
// //     msg!("   Original borrow: {}", loan.borrowed_underlying);
// //     msg!("   Debt shares (dTokens): {}", loan.borrowed_amount);
// //     msg!("   Current debt with interest: {}", debt_amount);


// //         // ========================================================================
// //     // STEP 3: Calculate repay fee
// //     // ========================================================================
// //     let repay_fee = debt_amount
// //         .checked_mul(borrow_market.repay_fee)
// //         .ok_or(LendingError::MathOverflow)?
// //         .checked_div(BASIS_POINTS)
// //         .ok_or(LendingError::MathOverflow)?;
    
// //     let total_repay = debt_amount
// //         .checked_add(repay_fee)
// //         .ok_or(LendingError::MathOverflow)?;

// //     require!(
// //         repay_amount >= total_repay,
// //         LendingError::RepayAmountTooSmall
// //     );


// //     let transfer_cpi_accounts = TransferChecked {
// //         from: ctx.accounts.user_token_account.to_account_info(),
// //         mint: ctx.accounts.mint.to_account_info(),
// //         to: ctx.accounts.supply_vault.to_account_info(),
// //         authority: ctx.accounts.borrower.to_account_info(),
// //     };

// //     let cpi_program = ctx.accounts.token_program.to_account_info();
// //     let cpi_ctx = CpiContext::new(cpi_program, transfer_cpi_accounts);
// //     let decimals = ctx.accounts.mint.decimals;

// //     token_interface::transfer_checked(cpi_ctx, total_repay, decimals)?;



// //     // ========================================================================
// //     // STEP 7: Unlock collateral
// //     // ========================================================================
// //     user_position.locked_collateral = user_position.locked_collateral
// //         .checked_sub(loan.collateral_amount)
// //         .ok_or(LendingError::MathOverflow)?;


// //     // ========================================================================
// //     // STEP 8: Update market and position states
// //     // ========================================================================
// //     borrow_market.total_borrows = borrow_market.total_borrows
// //         .checked_sub(debt_amount)
// //         .ok_or(LendingError::MathOverflow)?;

// //     borrow_market.total_borrowed_shares = borrow_market.total_borrowed_shares
// //         .checked_sub(loan.borrowed_amount)
// //         .ok_or(LendingError::MathOverflow)?;


// //         // ========================================================================
// //     // STEP 9: Update loan status
// //     // ========================================================================
// //     loan.status_u8 = 1; // Repaid
// //     loan.updated_at = clock.unix_timestamp;
    
// //     msg!("✅ Loan repaid successfully!");
// //     msg!("   Principal repaid: {}", debt_amount);
// //     msg!("   Interest paid: {}", debt_amount - loan.borrowed_underlying);
// //     msg!("   Fee: {}", repay_fee);
// //     msg!("   Total paid: {}", total_repay);
// //     msg!("   Collateral unlocked: {} rTokens", loan.collateral_amount);
    
    

    
    

// //     Ok(())
// // }





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
        seeds = [b"loan", borrower.key().as_ref(), &loan.loan_id.to_le_bytes()],
        bump = loan.bump,
        constraint = loan.borrower == borrower.key() @ LendingError::InvalidLoan,
    )]
    pub loan: Box<Account<'info, Loan>>, 

    // Protocol state
    #[account(
        seeds = [b"protocol_state"],
        bump = protocol_state.bump,
    )]
    pub protocol_state: Box<Account<'info, ProtocolState>>,  

    // collateral market to reduce lets see for now 
    #[account(
        mut,
        address = loan.collateral_market,
    )]
    pub collateral_market: Box<Account<'info, Market>>,  

    // Borrow market
    #[account(
        mut,
        address = loan.borrow_market,
    )]
    pub borrow_market: Box<Account<'info, Market>>, 

    #[account(
        mut,
        seeds = [b"user_position", borrower.key().as_ref(), collateral_market.key().as_ref()],
        bump = user_position.bump,
    )]
    pub user_position: Box<Account<'info, UserPosition>>,  // ✅ Boxed

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
        bump,
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
    // Logic for repaying the loan goes here

    let collateral_market = &mut ctx.accounts.collateral_market;
    let borrow_market = &mut ctx.accounts.borrow_market;
    let loan = &mut ctx.accounts.loan;
    let user_position = &mut ctx.accounts.user_position;

    let clock = Clock::get()?;

    // ========================================================================
    // STEP 1: Accrue interest to get current debt
    // ========================================================================
    accrue_interest(borrow_market, clock.unix_timestamp)?;

    let total_dtokens = borrow_market.total_borrowed_shares;
    let total_borrows = borrow_market.total_borrows;

    let debt_amount = (loan.borrowed_amount as u128)
        .checked_mul(total_borrows as u128)
        .ok_or(LendingError::MathOverflow)?
        .checked_div(total_dtokens as u128)
        .ok_or(LendingError::MathOverflow)?
        as u64;
    
    msg!("💰 Debt calculation:");
    msg!("   Original borrow: {}", loan.borrowed_underlying);
    msg!("   Debt shares (dTokens): {}", loan.borrowed_amount);
    msg!("   Current debt with interest: {}", debt_amount);

    // ========================================================================
    // STEP 2: Calculate repay fee
    // ========================================================================
    let repay_fee = debt_amount
        .checked_mul(borrow_market.repay_fee)
        .ok_or(LendingError::MathOverflow)?
        .checked_div(BASIS_POINTS)
        .ok_or(LendingError::MathOverflow)?;
    
    let total_repay = debt_amount
        .checked_add(repay_fee)
        .ok_or(LendingError::MathOverflow)?;

    require!(
        repay_amount >= total_repay,
        LendingError::RepayAmountTooSmall
    );

    // ========================================================================
    // STEP 3: Transfer repayment from user to vault
    // ========================================================================
    let transfer_cpi_accounts = TransferChecked {
        from: ctx.accounts.user_token_account.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.supply_vault.to_account_info(),
        authority: ctx.accounts.borrower.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, transfer_cpi_accounts);
    let decimals = ctx.accounts.mint.decimals;

    token_interface::transfer_checked(cpi_ctx, total_repay, decimals)?;

    // ========================================================================
    // STEP 4: Unlock collateral
    // ========================================================================
    user_position.locked_collateral = user_position.locked_collateral
        .checked_sub(loan.collateral_amount)
        .ok_or(LendingError::MathOverflow)?;

    // ========================================================================
    // STEP 5: Update market and position states
    // ========================================================================
    borrow_market.total_borrows = borrow_market.total_borrows
        .checked_sub(debt_amount)
        .ok_or(LendingError::MathOverflow)?;

    borrow_market.total_borrowed_shares = borrow_market.total_borrowed_shares
        .checked_sub(loan.borrowed_amount)
        .ok_or(LendingError::MathOverflow)?;

    // ========================================================================
    // STEP 6: Update loan status
    // ========================================================================
    // loan.status = crate::state::LoanStatus::Repaid;  // ✅ Use enum instead of u8
    loan.updated_at = clock.unix_timestamp;
    
    msg!("✅ Loan repaid successfully!");
    msg!("   Principal repaid: {}", debt_amount);
    msg!("   Interest paid: {}", debt_amount - loan.borrowed_underlying);
    msg!("   Fee: {}", repay_fee);
    msg!("   Total paid: {}", total_repay);
    msg!("   Collateral unlocked: {} rTokens", loan.collateral_amount);

    Ok(())
}