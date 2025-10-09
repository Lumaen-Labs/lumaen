use anchor_lang::prelude::*;
use switchboard_on_demand::{
    default_queue, Instructions, QueueAccountData, QuoteVerifier, SlotHashes,
};
switchboard_on_demand::switchboard_anchor_bindings!();

declare_id!("3AsHpu3rrzQjx1gTAWUKyiqaFj6HdSUFovLEhXpP2Ufv");

pub const DEFAULT_PRICE: i128 = -1;
pub const PRECISION: i128 = 1_000_000_000_000_000_000; // 1e18
pub const ANCHOR_DISCRIMINATOR_SIZE: usize = 8;

#[program]
pub mod pricer {
    use super::*;

    /// Initialize the pricer with Switchboard feed hash
    pub fn initialize(ctx: Context<Initialize>, feed_hash: String) -> Result<()> {
        let pricer = &mut ctx.accounts.pricer;
        pricer.authority = ctx.accounts.authority.key();
        pricer.feed_hash = feed_hash;
        Ok(())
    }

    /// Get asset base price from Switchboard oracle
    /// Returns the price of the asset, or DEFAULT_PRICE if price is 0
    pub fn get_asset_base_price(ctx: Context<GetPrice>) -> Result<i128> {
        // Validate that the quote account is the canonical account for our feed
        let expected_key = ctx.accounts.quote_account.canonical_key(&default_queue());
        require_keys_eq!(
            ctx.accounts.quote_account.key(),
            expected_key,
            ErrorCode::InvalidQuoteAccount
        );

        // Get the feeds from the quote account
        let feeds = ctx.accounts.quote_account.feeds();

        require!(!feeds.is_empty(), ErrorCode::NoFeedsAvailable);

        // Get the first feed's price
        let feed = &feeds[0];
        let price_f64 = feed.value();

        // Convert to i128 with 18 decimal precision
        let price = price_f64 as i128;
        let final_price = if price == 0 { DEFAULT_PRICE } else { price };

        msg!("Asset base price: {}", final_price);
        Ok(final_price)
    }

    /// Get asset base value (amount * price / 1e18)
    /// Calculates the USD value of a given token amount
    pub fn get_asset_base_value(ctx: Context<GetPrice>, amount: u64) -> Result<i128> {
        // Validate canonical account
        let expected_key = ctx.accounts.quote_account.canonical_key(&default_queue());
        require_keys_eq!(
            ctx.accounts.quote_account.key(),
            expected_key,
            ErrorCode::InvalidQuoteAccount
        );

        let feeds = ctx.accounts.quote_account.feeds();
        require!(!feeds.is_empty(), ErrorCode::NoFeedsAvailable);

        let feed = &feeds[0];
        let price_f64 = feed.value();
        let price = price_f64 as i128;
        let final_price = if price == 0 { DEFAULT_PRICE } else { price };

        let value = (amount as i128)
            .checked_mul(final_price)
            .ok_or(ErrorCode::MathOverflow)?
            .checked_div(PRECISION)
            .ok_or(ErrorCode::MathOverflow)?;

        msg!("Asset value for amount {}: {}", amount, value);
        Ok(value)
    }

    /// Get token amount from value (value * 1e18 / price)
    /// Calculates how many tokens you get for a given USD value
    pub fn get_token_amount_from_value(ctx: Context<GetPrice>, value: u64) -> Result<i128> {
        // Validate canonical account
        let expected_key = ctx.accounts.quote_account.canonical_key(&default_queue());
        require_keys_eq!(
            ctx.accounts.quote_account.key(),
            expected_key,
            ErrorCode::InvalidQuoteAccount
        );

        let feeds = ctx.accounts.quote_account.feeds();
        require!(!feeds.is_empty(), ErrorCode::NoFeedsAvailable);

        let feed = &feeds[0];
        let price_f64 = feed.value();
        let price = price_f64 as i128;
        let final_price = if price == 0 { DEFAULT_PRICE } else { price };

        let amount = (value as i128)
            .checked_mul(PRECISION)
            .ok_or(ErrorCode::MathOverflow)?
            .checked_div(final_price)
            .ok_or(ErrorCode::MathOverflow)?;

        msg!("Token amount for value {}: {}", value, amount);
        Ok(amount)
    }
}

#[account]
#[derive(InitSpace)]
pub struct Pricer {
    pub authority: Pubkey,
    #[max_len(64)]
    pub feed_hash: String,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = ANCHOR_DISCRIMINATOR_SIZE + Pricer::INIT_SPACE
    )]
    pub pricer: Account<'info, Pricer>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct GetPrice<'info> {
    pub pricer: Account<'info, Pricer>,

    /// The canonical oracle account containing verified quote data
    /// Validated to be the canonical account for the contained feeds
    #[account(address = quote_account.canonical_key(&default_queue()))]
    pub quote_account: InterfaceAccount<'info, SwitchboardQuote>,

    /// System variables required for quote verification
    pub sysvars: Sysvars<'info>,
}

/// System variables required for oracle verification
#[derive(Accounts)]
pub struct Sysvars<'info> {
    pub clock: Sysvar<'info, Clock>,
    pub slothashes: Sysvar<'info, SlotHashes>,
    pub instructions: Sysvar<'info, Instructions>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Math overflow occurred")]
    MathOverflow,
    #[msg("Invalid quote account")]
    InvalidQuoteAccount,
    #[msg("No feeds available in quote account")]
    NoFeedsAvailable,
}
