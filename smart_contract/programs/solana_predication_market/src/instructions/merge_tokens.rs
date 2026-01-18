use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Burn, Transfer};
use crate::state::Market;
use crate::error::PredictionMarketError;

#[derive(Accounts)]
#[instruction(market_id: u32)]
pub struct MergeToken<'info> {
    #[account(
        mut,
        seeds = [b"market", market.market_id.to_le_bytes().as_ref()],
        bump = market.bump,
        constraint = market.market_id == market_id
    )]
    pub market: Account<'info, Market>,
   
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        constraint = user_collateral.mint == market.collateral_mint,
        constraint = user_collateral.owner == user.key()
    )]
    pub user_collateral: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = collateral_vault.key() == market.collateral_vault,
        constraint = collateral_vault.owner == market.key(),
        constraint = collateral_vault.mint == market.collateral_mint,
    )]
    pub collateral_vault: Account<'info, TokenAccount>,

    #[account(
        constraint = outcome_a_mint.key() == market.outcome_a_mint
    )]
    pub outcome_a_mint: Account<'info, Mint>,
    
    #[account(
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

pub fn handle(
    ctx: Context<MergeToken>,
    _market_id: u32,
) -> Result<()> {
    let market = &mut ctx.accounts.market;

    require!(!market.is_settled, PredictionMarketError::MarketAlreadySettled);
    require!(
        Clock::get()?.unix_timestamp < market.settlement_deadline,
        PredictionMarketError::MarketExpired
    );

    let a_bal = ctx.accounts.user_outcome_a.amount;
    let b_bal = ctx.accounts.user_outcome_b.amount;

    let amount = a_bal.min(b_bal);

    require!(amount > 0, PredictionMarketError::InvalidAmount);

    // Burn outcome A tokens
    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.outcome_a_mint.to_account_info(),
                from: ctx.accounts.user_outcome_a.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        amount,
    )?;
    
    // Burn outcome B tokens
    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.outcome_b_mint.to_account_info(),
                from: ctx.accounts.user_outcome_b.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        amount,
    )?;
    
    let market_id_bytes = market.market_id.to_le_bytes();
    let seeds: &[&[u8]] = &[
        b"market",
        market_id_bytes.as_ref(),
        &[market.bump],
    ];
    let signer_seeds = &[&seeds[..]];
    
    // Transfer collateral back to user
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.collateral_vault.to_account_info(),
                to: ctx.accounts.user_collateral.to_account_info(),
                authority: market.to_account_info(),
            },
            signer_seeds,
        ),
        amount,
    )?;

    market.total_collateral_locked = market
        .total_collateral_locked
        .checked_sub(amount)
        .ok_or(PredictionMarketError::MathOverflow)?;

    msg!("Merged {} pairs of outcome tokens back to collateral", amount);
    Ok(())
}