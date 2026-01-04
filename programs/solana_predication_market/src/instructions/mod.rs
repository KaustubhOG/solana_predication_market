pub mod initialize_market;
pub mod split_tokens;
pub mod merge_tokens;
pub mod set_winning_side;
pub mod claim_rewards;

// Re-export both structs AND handler functions
pub use initialize_market::*;
pub use split_tokens::*;
pub use merge_tokens::*;
pub use set_winning_side::*;
pub use claim_rewards::*;