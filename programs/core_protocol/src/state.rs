use anchor_lang::prelude::*;

// Global protocol configuration (Single instance)
#[account]
#[derive(InitSpace)]
pub struct ProtocolState {
    pub admin: Pubkey,
    pub fee_collector: Pubkey,
    pub protocol_paused: bool,
    pub total_markets: u64,
    pub total_loans: u64,
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
    
    // Pyth price feed IDs
    // https://pyth.network/developers/price-feed-ids#solana-stable
    pub pyth_feed_id: [u8; 32],

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

// Remove InitSpace and calculate manually for structs with enums
#[account]
#[derive(InitSpace)]
pub struct Loan {

    pub borrower: Pubkey,
    pub loan_id: u64,          // Unique loan identifier will be an incremental number

    // Collateral
    pub collateral_market: Pubkey,       // underlying market like USDC, USDT
    pub collateral_amount: u64,          // rToken shares locked as collateral

    // Borrow
    pub borrow_market: Pubkey,           // underlying market like USDC, USDT    
    pub borrowed_amount: u64,            // dToken shares
    pub borrowed_underlying: u64,        // Actual asset amount borrowed (for tracking)

    // associated User Position
    pub user_position_account: Pubkey,   // User position account

    // L3 Integration (for spending loans in DeFi)
    pub current_market: Pubkey,          // Where is the loan currently?
    pub current_amount: u64,             // How much is there?
    pub l3_integration: Pubkey,          // Which protocol is it in?
    // pub current_spent_status: SpentStatus,
    pub current_spent_u8: u8,      // Additional info about current status
    // Status
    pub status_u8: u8,
    // pub status: LoanStatus,              // Active, Spent, Repaid, Liquidated
    pub created_at: i64,
    pub updated_at: i64,

    pub bump: u8,
}

impl Loan {
    // Manual space calculation:
    // 8 (discriminator) + 32 (borrower) + 8 (loan_id) + 
    // 32 (collateral_market) + 8 (collateral_amount) + 
    // 32 (borrow_market) + 8 (borrowed_amount) + 8 (borrowed_underlying) +
    // 32 (user_position_account) + 32 (current_market) + 8 (current_amount) +
    // 32 (l3_integration) + 1 (current_spent_status enum) + 1 (status enum) +
    // 8 (created_at) + 8 (updated_at) + 1 (bump)
    pub const LEN: usize = 8 + 32 + 8 + 32 + 8 + 32 + 8 + 8 + 32 + 32 + 8 + 32 + 1 + 1 + 8 + 8 + 1;
}

// impl Space for Loan {
//     const INIT_SPACE: usize = Self::LEN;
// }

// #[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq,InitSpace)]
// pub enum LoanStatus {
//     Active,      // Loan is active, funds with user
//     Spent,       // Loan funds spent in L3 protocol (DEX, etc.)
//     Repaid,      // Loan fully repaid
//     Liquidated,  // Loan was liquidated
// }

// #[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq,InitSpace)]
// pub enum SpentStatus {
//     NotSpent,
//     Deposit,         // Funds not yet spent
//     Withdraw,        // Funds spent in L3 protocol
//     AddLiquidity,    // Funds used to add liquidity
//     RemoveLiquidity, // Funds removed from liquidity
//     Swap,            // Funds swapped in DEX
//     GetL3Value,      // Fetch value from L3 protocol
//     Other,
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
}