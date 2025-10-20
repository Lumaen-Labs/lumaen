use anchor_lang::prelude::*;

#[error_code]
pub enum LendingError {
    #[msg("Deposit amount below minimum threshold")]
    DepositTooSmall,
    #[msg("Deposit amount exceeds maximum allowed")]
    DepositTooLarge,
    #[msg("Borrow amount below minimum threshold")]
    BorrowTooSmall,
    #[msg("Borrow amount exceeds maximum allowed")]
    BorrowTooLarge,
    #[msg("Insufficient liquidity in supply vault")]
    InsufficientLiquidity,
    #[msg("Loan-to-Value ratio exceeds maximum allowed")]
    LTVExceeded,
    #[msg("Health factor below liquidation threshold")]
    UnhealthyPosition,
    #[msg("Not enough unlocked collateral")]
    InsufficientCollateral,
    #[msg("Loan does not exist or not owned by user")]
    InvalidLoan,
    #[msg("Mathematical overflow occurred")]
    MathOverflow,
    #[msg("Market is paused")]
    MarketPaused,
    #[msg("Insufficient free rTokens to lock")]
    InsufficientFreeRTokens,
    #[msg("Withdraw exceeds daily limit (20% of reserves)")]
    WithdrawLimitExceeded,
    #[msg("Invalid market for the given asset")]
    InvalidMarket,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Repay amount exceeds outstanding borrow")]
    RepayAmountTooSmall,
}
