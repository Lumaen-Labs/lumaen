use anchor_lang::prelude::*;

declare_id!("3AsHpu3rrzQjx1gTAWUKyiqaFj6HdSUFovLEhXpP2Ufv");

pub mod instructions;
pub mod initialization;
pub mod state;
pub mod errors;
pub mod constants;
pub mod events;

use instructions::*;
use initialization::*;

#[program]
pub mod router {
    use super::*;

    // return type should be shares
    // pub fn deposit(ctx: Context<Deposit>,amount:u64) -> Result<u64> {
    //     instructions::deposit_handler(ctx,amount)
    // }

    pub fn initialize_protocol(ctx:Context<InitializeProtocol>)->Result<()>{
        initialization::handler_initialize_protocol(ctx)
    }
    pub fn 



}
