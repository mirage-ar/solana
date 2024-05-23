use anchor_lang::prelude::*;

#[error_code]
pub enum GGError {
    #[msg("You are not authorized to perform this action.")]
    Unauthorized,
    #[msg("Supply must be greater than 0.")]
    InvalidSupply,
    #[msg("You must pay at least 1 SOL to mint a token.")]
    InsufficientMintAmount,
    #[msg("Insufficient funds.")]
    InsufficientFunds,
    #[msg("Insufficient shares.")]
    InsufficientShares,
}
