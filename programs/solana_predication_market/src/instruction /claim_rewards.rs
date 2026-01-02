use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Burn, Transfer};
use crate::state::{Market, WinningOutcome};
use crate::error::PredictionMarketError;

#[derive(Accounts)]
#[instruction(market_id: u32)]
pub struct ClaimRewards<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"market", market.market_id.to_le_bytes().as_ref()],
        bump = market.bump,
        constraint = market.market_id == market_id
    )]
    pub market: Account<'info, Market>,

    #[account(
        mut,
        constraint = user_collateral.mint == market.collateral_mint,
        constraint = user_collateral.owner == user.key()
    )]
    pub user_collateral: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = collateral_vault.key() == market.collateral_vault
    )]
    pub collateral_vault: Account<'info, TokenAccount>,
     
    #[account(
        mut,
        constraint = outcome_a_mint.key() == market.outcome_a_mint
    )]
    pub outcome_a_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        constraint = outcome_b_mint.key() == market.outcome_b_mint
    )]
    pub outcome_b_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        constraint = user_outcome_a.mint == market.outcome_a_mint,
        constraint = user_outcome_a.owner == user.key()
    )]
    pub user_outcome_a: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = user_outcome_b.mint == market.outcome_b_mint,
        constraint = user_outcome_b.owner == user.key()
    )]
    pub user_outcome_b: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

pub fn handler(
    ctx: Context<ClaimRewards>,
    market_id: u32,
) -> Result<()> {
    let market = &mut ctx.accounts.market;

    require!(market.is_settled, PredictionMarketError::MarketNotSettled);

    let winner = market
        .winning_outcome
        .ok_or_else(|| error!(PredictionMarketError::WinningOutcomeNotSet))?;

    let (winner_mint_info, user_winner_ata) = match winner {
        WinningOutcome::OutcomeA => (
            ctx.accounts.outcome_a_mint.to_account_info(),
            &ctx.accounts.user_outcome_a,
        ),
        _ => (
            ctx.accounts.outcome_b_mint.to_account_info(),
            &ctx.accounts.user_outcome_b,
        ),
    };

    let amount = user_winner_ata.amount;

    // Burn winning tokens
    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: winner_mint_info,
                from: user_winner_ata.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        amount,
    )?;
    
    let market_id_bytes = market.market_id.to_le_bytes();
    let seeds = &[
        b"market",
        market_id_bytes.as_ref(),
        &[market.bump],
    ];
    
    // Transfer collateral to user
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.collateral_vault.to_account_info(),
                to: ctx.accounts.user_collateral.to_account_info(),
                authority: market.to_account_info(),
            },
            &[seeds],
        ),
        amount,
    )?;
    
    market.total_collateral_locked = market
        .total_collateral_locked
        .checked_sub(amount)
        .ok_or(PredictionMarketError::MathOverflow)?;

    msg!("Claimed {} collateral for winning side", amount);
    Ok(())
}