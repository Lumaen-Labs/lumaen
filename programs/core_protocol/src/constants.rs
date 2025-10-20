// ============================================================================
// PROGRAM CONSTANTS
// ============================================================================
pub const SECONDS_PER_YEAR: u64 = 31_536_000; // 365 days
pub const BASIS_POINTS: u64 = 10_000;
pub const PRECISION: u128 = 1_000_000_000_000_000_000; // 10^18
pub const MIN_HEALTH_FACTOR: u64 = 10_450; // 1.045 in basis points (10000 = 1.0)
pub const DAILY_WITHDRAW_LIMIT_BPS: u64 = 2_000; // 20%
pub const ANCHOR_DISCRIMINATOR_SIZE: usize = 8;
pub const MAXIMUM_AGE: u64 = 100; // allow price feed 100 sec old, to avoid stale price feed errors
