use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, MintTo, Transfer};
use crate::state::Market;
use crate::error::PredictionMarketError;

#[derive(Accounts)]
#[instruction(market_id: u32)]
pub struct SplitToken<'info> {
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
    ctx: Context<SplitToken>,
    market_id: u32,
    amount: u64,
) -> Result<()> {
    let market = &mut ctx.accounts.market;
    
    require!(!market.is_settled, PredictionMarketError::MarketAlreadySettled);
    require!(
        Clock::get()?.unix_timestamp < market.settlement_deadline,
        PredictionMarketError::MarketExpired
    );
    require!(amount > 0, PredictionMarketError::InvalidAmount);
    
    // Transfer collateral from user to vault
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_collateral.to_account_info(),
                to: ctx.accounts.collateral_vault.to_account_info(),
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
    let signer = &[&seeds[..]];
    
    // Mint outcome A tokens
    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.outcome_a_mint.to_account_info(),
                to: ctx.accounts.user_outcome_a.to_account_info(),
                authority: market.to_account_info(),
            },
            signer,
        ),
        amount,
    )?;
    
    // Mint outcome B tokens
    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.outcome_b_mint.to_account_info(),
                to: ctx.accounts.user_outcome_b.to_account_info(),
                authority: market.to_account_info(),
            },
            signer,
        ),
        amount,
    )?;
    
    market.total_collateral_locked = market.total_collateral_locked
        .checked_add(amount)
        .ok_or(PredictionMarketError::MathOverflow)?;
    
    msg!("Minted {} outcome tokens for user", amount);
    Ok(())
}