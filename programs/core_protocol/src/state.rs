use anchor_lang::prelude::*;

// Global protocol configuration (Single instance)
#[account]
#[derive(InitSpace)]
pub struct ProtocolState {
    pub admin: Pubkey,
    pub fee_collector: Pubkey,
    pub protocol_paused: bool,
    pub total_markets: u64,
    pub bump: u8,
}

/// Market configuration for each asset (SOL, USDC, etc.)
#[account]
#[derive(InitSpace)]
pub struct Market {
    pub mint: Pubkey,              // Underlying asset mint
    pub supply_vault: Pubkey,      // PDA holding deposits
    pub shares_token_mint: Pubkey, // Receipt token mint
    pub dtoken_mint: Pubkey,       // Debt token mint

    // Financial state
    pub total_deposits: u64, // Total underlying deposited
    pub total_deposited_shares: u64,
    pub total_borrowed_shares: u64,
    pub total_borrows: u64,  // Total underlying borrowed
    pub total_reserves: u64, // Protocol reserves

    // Interest tracking
    pub last_update_timestamp: i64,
    pub supply_index: u128, // Accumulated supply interest index
    pub borrow_index: u128, // Accumulated borrow interest index

    // Risk parameters
    pub max_ltv: u64,               // Max LTV in basis points (e.g., 7500 = 75%)
    pub liquidation_threshold: u64, // Liquidation threshold (e.g., 8000 = 80%)
    pub liquidation_penalty: u64,   // Liquidation bonus (e.g., 500 = 5%)
    pub reserve_factor: u64,        // % of interest to reserves (e.g., 1000 = 10%)

    // Limits
    pub min_deposit_amount: u64,
    pub max_deposit_amount: u64,
    pub min_borrow_amount: u64,
    pub max_borrow_amount: u64,

    // Daily withdraw limit tracking
    pub last_withdraw_reset_time: i64,
    pub deposit_snapshot: u64, // Snapshot at reset time

    // Fees (in basis points)
    pub deposit_fee: u64, // e.g., 10 = 0.1%
    pub withdraw_fee: u64,
    pub borrow_fee: u64,
    pub repay_fee: u64,

    pub paused: bool,
    pub bump: u8,
}

/// User's position in a specific market
#[account]
#[derive(InitSpace)]
pub struct UserPosition {
    pub user: Pubkey,
    pub market: Pubkey, // underlying market

    // Deposit tracking
    pub deposited_shares: u64,  // rToken shares owned
    pub locked_collateral: u64, // rToken shares locked as collateral

    // Borrow tracking
    pub borrowed_shares: u64, // dToken shares (debt)

    // Interest tracking (for accurate calculations)
    pub deposit_index: u128, // Last supply index when user interacted
    pub borrow_index: u128,  // Last borrow index when user interacted

    pub bump: u8,
}

impl UserPosition {
    pub fn free_rtokens(&self) -> u64 {
        self.deposited_shares.checked_sub(self.locked_collateral).expect("ShutUp")
    }
}

// #[account]
// pub struct Loan {
//     pub borrower: Pubkey,
//     pub loan_id: u64,

//     // Collateral
//     pub collateral_market: Pubkey,
//     pub collateral_amount: u64,          // rToken shares locked

//     // Borrow
//     pub borrow_market: Pubkey,
//     pub borrowed_amount: u64,            // dToken shares
//     pub borrowed_underlying: u64,        // Actual asset amount borrowed (for tracking)

//     // L3 Integration (for spending loans in DeFi)
//     pub current_market: Pubkey,          // Where is the loan currently?
//     pub current_amount: u64,             // How much is there?
//     pub l3_integration: Pubkey,          // Which protocol is it in?

//     // Status
//     pub status: LoanStatus,              // Active, Spent, Repaid, Liquidated
//     pub created_at: i64,
//     pub updated_at: i64,

//     pub bump: u8,
// }

// impl Loan {
//     pub const LEN: usize = 8 + 32 + 8 + 32*2 + 8 + 32 + 8*2 + 32*3 + 1 + 8*2 + 1;
// }

// #[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
// pub enum LoanStatus {
//     Active,      // Loan is active, funds with user
//     Spent,       // Loan funds spent in L3 protocol (DEX, etc.)
//     Repaid,      // Loan fully repaid
//     Liquidated,  // Loan was liquidated
// }



#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct MarketConfig {
    // Risk parameters
    pub max_ltv: u64,               // e.g., 7500 = 75%
    pub liquidation_threshold: u64, // e.g., 8000 = 80%
    pub liquidation_penalty: u64,   // e.g., 500 = 5%
    pub reserve_factor: u64,        // e.g., 1000 = 10%

    // Limits
    pub min_deposit_amount: u64,
    pub max_deposit_amount: u64,
    pub min_borrow_amount: u64,
    pub max_borrow_amount: u64,

    // Fees (basis points)
    pub deposit_fee: u64, // e.g., 10 = 0.1%
    pub withdraw_fee: u64,
    pub borrow_fee: u64,
    pub repay_fee: u64,
    // Interest rate model
    // pub base_rate: u64,                  // e.g., 200 = 2%
    // pub optimal_utilization: u64,        // e.g., 8000 = 80%
    // pub slope1: u64,                     // e.g., 400 = 4%
    // pub slope2: u64,                     // e.g., 7500 = 75%
}

