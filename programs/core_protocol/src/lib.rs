use anchor_lang::prelude::*;

declare_id!("3AsHpu3rrzQjx1gTAWUKyiqaFj6HdSUFovLEhXpP2Ufv");

#[program]
pub mod router {
    use super::*;

    // return type should be shares
    pub fn deposit(ctx: Context<Deposit>) -> Result<u256> {
        Ok((0))
    }

    pub fn supply(ctx: Context<Deposit>) -> Result<u256> {
        Ok((0))
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<u256> {
        Ok((0))
    }

    // return type should be a loanId
    pub fn borrow(ctx: Context<Borrow>) -> Result<u256> {
        Ok((0))
    }

    pub fn borrow_with_r_token(ctx: Context<BorrowWithRToken>) -> Result<u256> {
        Ok((0))
    }

    pub fn add_collateral(ctx: Context<AddCollateral>) -> Result<()> {
        Ok(())
    }

    pub fn add_r_token_collateral(ctx: Context<AddRTokenCollateral>) -> Result<()> {
        Ok(())
    }

    pub fn borrow_and_spend(ctx: Context<BorrowAndSpend>) -> Result<(LoanSpent)> {
        Ok(())
    }

    pub fn borrow_and_spend_with_r_token(
        ctx: Context<BorrowAndSpendWithRToken>,
    ) -> Result<(LoanSpent)> {
        Ok(())
    }

    pub fn repay_loan(ctx: Context<RepayLoan>) -> Result<AssetReleased> {
        Ok(())
    }

    pub fn spend_loan(ctx: Context<SpendLoan>) -> Result<LoanSpent> {
        Ok(())
    }

    pub fn revert_spent_loan(ctx: Context<RevertSpentLoan>) -> Result<RevertLoanResult> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct RevertSpentLoan<'info> {}

#[derive(Accounts)]
pub struct Deposit<'info> {}

#[derive(Accounts)]
pub struct Withdraw<'info> {}

#[derive(Accounts)]
pub struct Borrow<'info> {}

#[derive(Accounts)]
pub struct BorrowWithRToken<'info> {}

#[derive(Accounts)]
pub struct AddCollateral<'info> {}

#[derive(Accounts)]
pub struct AddRTokenCollateral<'info> {}

#[derive(Accounts)]
pub struct BorrowAndSpend<'info> {}

#[derive(Accounts)]
pub struct BorrowAndSpendWithRToken<'info> {}

#[derive(Accounts)]
pub struct RepayLoan<'info> {}

#[derive(Accounts)]
pub struct SpendLoan<'info> {}
