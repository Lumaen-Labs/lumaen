use anchor_lang::prelude::*;

declare_id!("3AsHpu3rrzQjx1gTAWUKyiqaFj6HdSUFovLEhXpP2Ufv");

#[program]
pub mod second {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
