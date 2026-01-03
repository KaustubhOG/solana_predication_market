use anchor_lang::prelude::*;

pub mod error;
pub mod instructions;
pub mod state;

declare_id!("J3kZR7H8pqE67JELXpq4BRP7Ws7JWDCvxHE5UwVZyw2L");

#[program]
pub mod prediction_market {
    use super::*;

    pub fn initialize_market(
        ctx: Context<instructions::InitializeMarket>,
        market_id: u32,
        settlement_deadline: i64,
    ) -> Result<()> {
        instructions::initialize_market::handle(ctx, market_id, settlement_deadline)
    }

    pub fn split_tokens(
        ctx: Context<instructions::SplitToken>,
        market_id: u32,
        amount: u64,
    ) -> Result<()> {
        instructions::split_tokens::handle(ctx, market_id, amount)
    }

    pub fn merge_tokens(
        ctx: Context<instructions::MergeToken>,
        market_id: u32,
    ) -> Result<()> {
        instructions::merge_tokens::handle(ctx, market_id)
    }

    pub fn set_winning_side(
        ctx: Context<instructions::SetWinner>,
        market_id: u32,
        winner: state::WinningOutcome,
    ) -> Result<()> {
        instructions::set_winning_side::handle(ctx, market_id, winner)
    }

    pub fn claim_rewards(
        ctx: Context<instructions::ClaimRewards>,
        market_id: u32,
    ) -> Result<()> {
        instructions::claim_rewards::handle(ctx, market_id)
    }
}