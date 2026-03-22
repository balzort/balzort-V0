use crate::states::Player;
use crate::types::constants::*;
use crate::types::GameError;
use anchor_lang::prelude::*;

pub fn handle_open_session(
    ctx: Context<OpenSession>,
    session_key: Pubkey,
    expires_in_secs: u32,
) -> Result<()> {
    let clock = Clock::get()?;
    require!(
        (expires_in_secs as i64) >= MIN_SESSION_DURATION_SECS, GameError::InvalidInput
    );
    require!(
        (expires_in_secs as i64) <= MAX_SESSION_DURATION_SECS, GameError::InvalidInput
    );

    let auth = &mut ctx.accounts.player;

    if let Some(_old_key) = auth.session_key {
        if clock.unix_timestamp < auth.session_expires_at {}
    }

    auth.session_key = Some(session_key);
    auth.session_expires_at = clock.unix_timestamp + expires_in_secs as i64;

    Ok(())
}

#[derive(Accounts)]
pub struct OpenSession<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut, seeds=[SEED_PLAYER, signer.key().as_ref()], bump=player.bump,
              constraint = player.wallet == signer.key() @ GameError::Unauthorized)]
    pub player: Account<'info, Player>,
}



