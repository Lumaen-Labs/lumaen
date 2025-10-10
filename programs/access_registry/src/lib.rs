use anchor_lang::prelude::*;
use anchor_lang::solana_program::keccak;

declare_id!("3AsHpu3rrzQjx1gTAWUKyiqaFj6HdSUFovLEhXpP2Ufv");

// Role definitions
pub const ADMIN_ROLE: &[u8] = b"ADMIN_ROLE";
pub const ANCHOR_DISCRIMINATOR_SIZE: usize = 8;

#[program]
pub mod access_registry {
    use super::*;

    /// Initialize the access registry with an authority
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let registry = &mut ctx.accounts.registry;
        registry.authority = ctx.accounts.authority.key();
        Ok(())
    }

    /// Grant a role to an address
    pub fn grant_role(ctx: Context<GrantRole>, role: Vec<u8>) -> Result<()> {
        let role_account = &mut ctx.accounts.role_account;
        role_account.role = role.clone();
        role_account.account = ctx.accounts.target.key();
        role_account.granted_by = ctx.accounts.authority.key();
        emit!(RoleGranted {
            role,
            account: ctx.accounts.target.key(),
            granted_by: ctx.accounts.authority.key()
        });
        Ok(())
    }

    /// Revoke a role from an address
    pub fn revoke_role(ctx: Context<RevokeRole>) -> Result<()> {
        emit!(RoleRevoked {
            role: ctx.accounts.role_account.role.clone(),
            account: ctx.accounts.target.key(),
            revoked_by: ctx.accounts.authority.key()
        });
        Ok(())
    }

    /// Renounce a role (self-revoke)
    pub fn renounce_role(ctx: Context<RenounceRole>) -> Result<()> {
        emit!(RoleRenounced {
            role: ctx.accounts.role_account.role.clone(),
            account: ctx.accounts.signer.key(),
            renounced_by: ctx.accounts.signer.key()
        });
        Ok(())
    }

    /// Transfer authority role to a new address
    pub fn transfer_authority(ctx: Context<TransferAuthority>) -> Result<()> {
        let registry = &mut ctx.accounts.registry;
        let old_authority = registry.authority;
        registry.authority = ctx.accounts.new_authority.key();

        emit!(RoleAuthorityTransferred {
            role: ADMIN_ROLE.to_vec(),
            account: old_authority,
            authority: ctx.accounts.new_authority.key()
        });
        Ok(())
    }

    /// Check if an account has a specific role
    pub fn has_role(ctx: Context<HasRole>) -> Result<bool> {
        // If the role_account PDA exists and validation passes, the account has the role
        Ok(true)
    }

    pub fn get_authority(ctx: Context<GetAuthority>) -> Result<Pubkey> {
        Ok(ctx.accounts.registry.authority)
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = ANCHOR_DISCRIMINATOR_SIZE + Registry::INIT_SPACE
    )]
    pub registry: Account<'info, Registry>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(role: Vec<u8>)]
pub struct GrantRole<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        has_one = authority @ AccessError::Unauthorized
    )]
    pub registry: Account<'info, Registry>,

    /// CHECK: The account receiving the role
    pub target: AccountInfo<'info>,

    #[account(
        init,
        payer = authority,
        space = ANCHOR_DISCRIMINATOR_SIZE + RoleAccount::INIT_SPACE,
        seeds = [b"role", registry.key().as_ref(), role.as_ref(), target.key().as_ref()],
        bump
    )]
    pub role_account: Account<'info, RoleAccount>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RevokeRole<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        has_one = authority @ AccessError::Unauthorized
    )]
    pub registry: Account<'info, Registry>,

    /// CHECK: The account losing the role
    pub target: AccountInfo<'info>,

    #[account(
        mut,
        close = authority,
        seeds = [b"role", registry.key().as_ref(), role_account.role.as_ref(), target.key().as_ref()],
        bump
    )]
    pub role_account: Account<'info, RoleAccount>,
}

#[derive(Accounts)]
pub struct RenounceRole<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub registry: Account<'info, Registry>,

    #[account(
        mut,
        close = signer,
        seeds = [b"role", registry.key().as_ref(), role_account.role.as_ref(), signer.key().as_ref()],
        bump,
        constraint = role_account.account == signer.key() @ AccessError::Unauthorized
    )]
    pub role_account: Account<'info, RoleAccount>,
}

#[derive(Accounts)]
pub struct TransferAuthority<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = authority @ AccessError::Unauthorized
    )]
    pub registry: Account<'info, Registry>,

    /// CHECK: The new authority account
    pub new_authority: AccountInfo<'info>,
}

// Context for checking if an account has a role
#[derive(Accounts)]
#[instruction(role: Vec<u8>)]
pub struct HasRole<'info> {
    pub registry: Account<'info, Registry>,

    /// CHECK: The account to check
    pub account: AccountInfo<'info>,

    #[account(
        seeds = [b"role", registry.key().as_ref(), role.as_ref(), account.key().as_ref()],
        bump
    )]
    pub role_account: Account<'info, RoleAccount>,
}

#[derive(Accounts)]
pub struct GetAuthority<'info> {
    pub registry: Account<'info, Registry>,
}

#[account]
#[derive(InitSpace)]
pub struct Registry {
    pub authority: Pubkey,
}

#[account]
#[derive(InitSpace)]
pub struct RoleAccount {
    #[max_len(32)]
    pub role: Vec<u8>,
    pub account: Pubkey,
    pub granted_by: Pubkey,
}

#[error_code]
pub enum AccessError {
    #[msg("Unauthorized: caller does not have required permissions")]
    Unauthorized,
    #[msg("Invalid role")]
    InvalidRole,
}

#[event]
#[derive(Debug)]
pub struct RoleGranted {
    pub role: Vec<u8>,
    pub account: Pubkey,
    pub granted_by: Pubkey,
}

#[event]
#[derive(Debug)]
pub struct RoleRevoked {
    pub role: Vec<u8>,
    pub account: Pubkey,
    pub revoked_by: Pubkey,
}

#[event]
#[derive(Debug)]
pub struct RoleRenounced {
    pub role: Vec<u8>,
    pub account: Pubkey,
    pub renounced_by: Pubkey,
}

#[event]
#[derive(Debug)]
pub struct RoleAuthorityTransferred {
    pub role: Vec<u8>,
    pub account: Pubkey,
    pub authority: Pubkey,
}
