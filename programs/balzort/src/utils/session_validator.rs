use crate::states::Player;
use crate::types::GameError;
use anchor_lang::prelude::*;

pub fn validate_signer(signer: &Pubkey, auth: &Player, now: i64) -> Result<()> {
    require!(auth.is_valid_signer(signer, now), GameError::Unauthorized);
    Ok(())
}

