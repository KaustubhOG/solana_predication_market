use anchor_lang::prelude::*;

pub mod error;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("J3kZR7H8pqE67JELXpq4BRP7Ws7JWDCvxHE5UwVZyw2L");

#[program]
pub mod prediction_market {
    use super::*;

    pub fn initialize_market(
        ctx: Context<InitializeMarket>,
        market_id: u32,
        settlement_deadline: i64,
    ) -> Result<()> {
        initialize_market::handler(ctx, market_id, settlement_deadline)
    }

    pub fn split_tokens(ctx: Context<SplitToken>, market_id: u32, amount: u64) -> Result<()> {
        split_tokens::handler(ctx, market_id, amount)
    }

    pub fn merge_tokens(ctx: Context<MergeToken>, market_id: u32) -> Result<()> {
        merge_tokens::handler(ctx, market_id)
    }

    pub fn set_winning_side(
        ctx: Context<SetWinner>,
        market_id: u32,
        winner: state::WinningOutcome,
    ) -> Result<()> {
        set_winning_side::handler(ctx, market_id, winner)
    }

    pub fn claim_rewards(ctx: Context<ClaimRewards>, market_id: u32) -> Result<()> {
        claim_rewards::handler(ctx, market_id)
    }
}
