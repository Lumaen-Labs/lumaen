use crate::constants::PRECISION;
use crate::errors::LendingError;
use crate::state::Market;
use anchor_lang::prelude::*;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Accrues interest on a market based on time elapsed and utilization
pub fn accrue_interest(market: &mut Market, current_timestamp: i64) -> Result<()> {
    let time_elapsed = current_timestamp
        .checked_sub(market.last_update_timestamp)
        .ok_or(LendingError::MathOverflow)?;

    if time_elapsed == 0 {
        return Ok(()); // No time passed
    }

    // ========================================================================
    // Calculate utilization rate
    // ========================================================================
    if market.total_deposits == 0 {
        market.last_update_timestamp = current_timestamp;
        return Ok(()); // No deposits yet
    }

    let utilization = (market.total_borrows as u128)
        .checked_mul(PRECISION)
        .ok_or(LendingError::MathOverflow)?
        .checked_div(market.total_deposits as u128)
        .ok_or(LendingError::MathOverflow)?;

    // ========================================================================
    // TODO: INTEREST RATE MODEL
    // ========================================================================
    // This is where you'd integrate with a sophisticated interest rate model
    // For now, using a simple linear model:
    //
    // Base Rate: 2% APR
    // Optimal Utilization: 80%
    // Rate Slope 1 (below optimal): 4% per utilization point
    // Rate Slope 2 (above optimal): 75% per utilization point
    //
    // This creates a "kink" model similar to Aave/Compound

    let base_rate = 200u128; // 2% in basis points (200/10000)
    let optimal_util = (80u128 * PRECISION) / 100; // 80%

    let borrow_rate_annual = if utilization <= optimal_util {
        // Below optimal: gentle slope
        let rate_at_optimal = 3200u128; // 32% at optimal
        base_rate + (rate_at_optimal - base_rate) * utilization / optimal_util
    } else {
        // Above optimal: steep slope
        let excess_util = utilization - optimal_util;
        let max_excess = PRECISION - optimal_util;
        3200u128 + (7500u128 * excess_util / max_excess) // Up to 107% at 100% util
    };

    // Convert annual rate to per-second rate
    let borrow_rate_per_second = borrow_rate_annual
        .checked_mul(PRECISION)
        .ok_or(LendingError::MathOverflow)?
        .checked_div((SECONDS_PER_YEAR as u128) * (BASIS_POINTS as u128))
        .ok_or(LendingError::MathOverflow)?;

    // ========================================================================
    // Calculate interest accrued
    // ========================================================================
    let interest = (market.total_borrows as u128)
        .checked_mul(borrow_rate_per_second)
        .ok_or(LendingError::MathOverflow)?
        .checked_mul(time_elapsed as u128)
        .ok_or(LendingError::MathOverflow)?
        .checked_div(PRECISION)
        .ok_or(LendingError::MathOverflow)? as u64;

    // ========================================================================
    // Split interest: reserves vs suppliers
    // ========================================================================
    let to_reserves = interest
        .checked_mul(market.reserve_factor)
        .ok_or(LendingError::MathOverflow)?
        .checked_div(BASIS_POINTS)
        .ok_or(LendingError::MathOverflow)?;

    let to_suppliers = interest
        .checked_sub(to_reserves)
        .ok_or(LendingError::MathOverflow)?;

    // ========================================================================
    // Update market state
    // ========================================================================
    market.total_borrows = market
        .total_borrows
        .checked_add(interest)
        .ok_or(LendingError::MathOverflow)?;

    market.total_deposits = market
        .total_deposits
        .checked_add(to_suppliers)
        .ok_or(LendingError::MathOverflow)?;

    market.total_reserves = market
        .total_reserves
        .checked_add(to_reserves)
        .ok_or(LendingError::MathOverflow)?;

    // Update indices (used for per-user interest tracking)
    let borrow_index_delta = (market.borrow_index)
        .checked_mul(borrow_rate_per_second)
        .ok_or(LendingError::MathOverflow)?
        .checked_mul(time_elapsed as u128)
        .ok_or(LendingError::MathOverflow)?
        .checked_div(PRECISION)
        .ok_or(LendingError::MathOverflow)?;

    market.borrow_index = market
        .borrow_index
        .checked_add(borrow_index_delta)
        .ok_or(LendingError::MathOverflow)?;

    // Supply index grows slower (due to reserve factor)
    let supply_rate_per_second = borrow_rate_per_second
        .checked_mul(utilization)
        .ok_or(LendingError::MathOverflow)?
        .checked_div(PRECISION)
        .ok_or(LendingError::MathOverflow)?
        .checked_mul((BASIS_POINTS - market.reserve_factor) as u128)
        .ok_or(LendingError::MathOverflow)?
        .checked_div(BASIS_POINTS as u128)
        .ok_or(LendingError::MathOverflow)?;

    let supply_index_delta = (market.supply_index)
        .checked_mul(supply_rate_per_second)
        .ok_or(LendingError::MathOverflow)?
        .checked_mul(time_elapsed as u128)
        .ok_or(LendingError::MathOverflow)?
        .checked_div(PRECISION)
        .ok_or(LendingError::MathOverflow)?;

    market.supply_index = market
        .supply_index
        .checked_add(supply_index_delta)
        .ok_or(LendingError::MathOverflow)?;

    market.last_update_timestamp = current_timestamp;

    msg!("💰 Interest accrued:");
    msg!("   Time elapsed: {}s", time_elapsed);
    msg!("   Utilization: {}%", (utilization * 100 / PRECISION));
    msg!("   Borrow APR: {}%", (borrow_rate_annual / 100));
    msg!("   Interest: {}", interest);
    msg!("   To reserves: {}", to_reserves);
    msg!("   To suppliers: {}", to_suppliers);
    Ok(())
}
