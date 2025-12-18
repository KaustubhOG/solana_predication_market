use anchor_lang::prelude::*;

declare_id!("J3kZR7H8pqE67JELXpq4BRP7Ws7JWDCvxHE5UwVZyw2L");

#[program]
pub mod solana_predication_market {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
