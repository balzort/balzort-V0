use crate::states::Player;
use crate::types::constants::*;
use crate::types::GameError;
use anchor_lang::prelude::*;

pub fn handle_close_session(ctx: Context<CloseSession>) -> Result<()> {
    let auth = &mut ctx.accounts.player;
    auth.session_key = None;
    auth.session_expires_at = 0;
    Ok(())
}

#[derive(Accounts)]
pub struct CloseSession<'info> {
    pub signer: Signer<'info>,
    #[account(mut, seeds=[SEED_PLAYER, signer.key().as_ref()], bump=player.bump,
              constraint = player.wallet == signer.key() @ GameError::Unauthorized)]
    pub player: Account<'info, Player>,
}




