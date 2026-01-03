pub mod initialize_market;
pub mod split_tokens;
pub mod merge_tokens;
pub mod set_winning_side;
pub mod claim_rewards;

// Only re-export the structs, not everything
pub use initialize_market::InitializeMarket;
pub use split_tokens::SplitToken;
pub use merge_tokens::MergeToken;
pub use set_winning_side::SetWinner;
pub use claim_rewards::ClaimRewards;