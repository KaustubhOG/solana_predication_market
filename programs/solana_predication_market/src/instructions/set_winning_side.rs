use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, SetAuthority, spl_token::instruction::AuthorityType};
use crate::state::{Market, WinningOutcome};
use crate::error::PredictionMarketError;

#[derive(Accounts)]
#[instruction(market_id: u32)]
pub struct SetWinner<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"market", market.market_id.to_le_bytes().as_ref()],
        bump = market.bump,
        constraint = market.market_id == market_id,
        constraint = market.authority == authority.key()
    )]
    pub market: Account<'info, Market>,

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
    
    pub token_program: Program<'info, Token>,
}

pub fn handler(
    ctx: Context<SetWinner>,
    market_id: u32,
    winner: WinningOutcome,
) -> Result<()> {
    let market = &mut ctx.accounts.market;

    require!(!market.is_settled, PredictionMarketError::MarketAlreadySettled);
    
    require!(
        Clock::get()?.unix_timestamp < market.settlement_deadline,
        PredictionMarketError::MarketExpired
    );

    require!(
        matches!(winner, WinningOutcome::OutcomeA | WinningOutcome::OutcomeB),
        PredictionMarketError::InvalidWinningOutcome
    );
    
    market.is_settled = true;
    market.winning_outcome = Some(winner);

    let market_id_bytes = market.market_id.to_le_bytes();
    let seeds = &[
        b"market",
        market_id_bytes.as_ref(),
        &[market.bump],
    ];
    let signer = &[&seeds[..]];

    // Remove mint authority from outcome A
    token::set_authority(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            SetAuthority {
                account_or_mint: ctx.accounts.outcome_a_mint.to_account_info(),
                current_authority: market.to_account_info(),
            },
            signer,
        ),
        AuthorityType::MintTokens,
        None,
    )?;

    // Remove mint authority from outcome B
    token::set_authority(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            SetAuthority {
                account_or_mint: ctx.accounts.outcome_b_mint.to_account_info(),
                current_authority: market.to_account_info(),
            },
            signer,
        ),
        AuthorityType::MintTokens,
        None,
    )?;
    
    msg!("Market settled. Winner: {:?}", winner);
    Ok(())
}