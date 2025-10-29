use anchor_lang::prelude::*;

// ========== INSTRUCTIONS ==========

pub fn update_credit_score_handler(
    ctx: Context<UpdateCreditScore>,
    user_pubkey: Pubkey,
    new_score: u16,
    proof_hash: [u8; 32],
    expiry: i64,
) -> Result<()> {
    let credit_account = &mut ctx.accounts.credit_score_account;
    let clock = Clock::get()?;

    // Validate score range
    require!(new_score <= 1000, CreditScoreError::InvalidScore);

    // Validate expiry is in the future
    require!(
        expiry > clock.unix_timestamp,
        CreditScoreError::InvalidExpiry
    );

    // If account already exists, enforce never-decrease rule
    if credit_account.score > 0 {
        require!(
            new_score >= credit_account.score,
            CreditScoreError::ScoreCannotDecrease
        );
    }

    // Update account data
    credit_account.user = user_pubkey;
    credit_account.score = new_score;
    credit_account.proof_hash = proof_hash;
    credit_account.last_updated = clock.unix_timestamp;
    credit_account.expiry = expiry;
    credit_account.bump = ctx.bumps.credit_score_account;

    msg!(
        "Updated credit score for {}: {})",
        user_pubkey,
        new_score
    );

    Ok(())
}

pub fn get_credit_score_handler(ctx: Context<GetCreditScore>) -> Result<u16> {
    let credit_account = &ctx.accounts.credit_score_account;
    let clock = Clock::get()?;

    // Check if score is expired
    if clock.unix_timestamp > credit_account.expiry {
        msg!("Warning: Credit score expired");
        return Ok(0);
    }

    Ok(credit_account.score)
}

// ========== ACCOUNTS ==========

#[derive(Accounts)]
#[instruction(user_pubkey: Pubkey)]
pub struct UpdateCreditScore<'info> {
    #[account(
        init_if_needed,
        payer = authority,
        space = 8 + CreditScoreAccount::INIT_SPACE,
        seeds = [b"credit_score", user_pubkey.as_ref()],
        bump
    )]
    pub credit_score_account: Account<'info, CreditScoreAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct GetCreditScore<'info> {
    #[account(
        seeds = [b"credit_score", credit_score_account.user.as_ref()],
        bump = credit_score_account.bump
    )]
    pub credit_score_account: Account<'info, CreditScoreAccount>,
}

// ========== STATE ==========

#[account]
#[derive(InitSpace)]
pub struct CreditScoreAccount {
    pub user: Pubkey,         // 32 bytes
    pub score: u16,           // 2 bytes
    pub proof_hash: [u8; 32], // 32 bytes
    pub last_updated: i64,    // 8 bytes
    pub expiry: i64,          // 8 bytes
    pub bump: u8,             // 1 byte
}

// ========== ERRORS ==========

#[error_code]
pub enum CreditScoreError {
    #[msg("Invalid score value; must be 0–1000")]
    InvalidScore,
    #[msg("Expiry timestamp must be in the future")]
    InvalidExpiry,
    #[msg("Credit score cannot decrease")]
    ScoreCannotDecrease,
}


// Score Range -> Max Leverage
//   800-1000  -> 5x
//   600-799   -> 4x
//   400-599   -> 3x
//   200-399   -> 2x
//   0-199     -> 1.5x