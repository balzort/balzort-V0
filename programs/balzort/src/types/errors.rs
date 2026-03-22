use anchor_lang::prelude::*;

#[error_code]
pub enum GameError {
    #[msg("Unauthorized signer — not the player wallet or a valid session key")]
    Unauthorized,
    #[msg("Only the authority can call this")]
    NotAuthority,
    #[msg("Only the tournament authority can call this")]
    NotTournamentAuthority,
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
    #[msg("Invalid state for this operation")]
    InvalidState,
    #[msg("Invalid input parameter")]
    InvalidInput,
}
