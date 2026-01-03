use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::state::Market;
use crate::error::PredictionMarketError;

#[derive(Accounts)]
#[instruction(market_id: u32)]
pub struct InitializeMarket<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Market::INIT_SPACE,
        seeds = [b"market", market_id.to_le_bytes().as_ref()],
        bump
    )]
    pub market: Account<'info, Market>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub collateral_mint: Account<'info, Mint>,
    
    #[account(
        init,
        payer = authority,
        token::mint = collateral_mint,
        token::authority = market,
        seeds = [b"vault", market_id.to_le_bytes().as_ref()],
        bump
    )]
    pub collateral_vault: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = authority,
        mint::decimals = 6,
        mint::authority = market,
        seeds = [b"outcome_a", market_id.to_le_bytes().as_ref()],
        bump
    )]
    pub outcome_a_mint: Account<'info, Mint>,
    
    #[account(
        init,
        payer = authority,
        mint::decimals = 6,
        mint::authority = market,
        seeds = [b"outcome_b", market_id.to_le_bytes().as_ref()],
        bump
    )]
    pub outcome_b_mint: Account<'info, Mint>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(
    ctx: Context<InitializeMarket>,
    market_id: u32,
    settlement_deadline: i64,
) -> Result<()> {
    let market = &mut ctx.accounts.market;
    
    require!(
        settlement_deadline > Clock::get()?.unix_timestamp,
        PredictionMarketError::InvalidSettlementDeadline
    );
    
    market.authority = ctx.accounts.authority.key();
    market.market_id = market_id;
    market.settlement_deadline = settlement_deadline;
    market.outcome_a_mint = ctx.accounts.outcome_a_mint.key();
    market.outcome_b_mint = ctx.accounts.outcome_b_mint.key();
    market.collateral_mint = ctx.accounts.collateral_mint.key();
    market.collateral_vault = ctx.accounts.collateral_vault.key();
    market.is_settled = false;
    market.winning_outcome = None;
    market.total_collateral_locked = 0;
    market.bump = ctx.bumps.market;
    
    msg!("Market initialized: {}", market.market_id);
    Ok(())
}