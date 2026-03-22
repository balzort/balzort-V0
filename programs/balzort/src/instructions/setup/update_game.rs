use crate::states::Game;
use crate::types::constants::*;
use crate::types::{GameError, GameUpdated};
use anchor_lang::prelude::*;

pub fn handle_update_game(
    ctx: Context<UpdateGame>,
    p: UpdateGameParams,
) -> Result<()> {
    let cfg = &mut ctx.accounts.game;

    if let Some(v) = p.treasury {
        cfg.treasury = v;
        emit!(GameUpdated {
            authority: ctx.accounts.authority.key(),
            field: "treasury".into(),
        });
    }

    if let Some(v) = p.is_paused {
        cfg.is_paused = v;
        emit!(GameUpdated {
            authority: ctx.accounts.authority.key(),
            field: "is_paused".into(),
        });
    }
    if let Some(v) = p.treasury_fee_bps {
        require!(v <= MAX_TREASURY_FEE_BPS, GameError::InvalidInput);
        cfg.treasury_fee_bps = v;
        emit!(GameUpdated {
            authority: ctx.accounts.authority.key(),
            field: "treasury_fee_bps".into(),
        });
    }
    Ok(())
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UpdateGameParams {
    pub treasury: Option<Pubkey>,
    pub treasury_fee_bps: Option<u16>,
    pub is_paused: Option<bool>,
}

#[derive(Accounts)]
pub struct UpdateGame<'info> {
    pub authority: Signer<'info>,
    #[account(mut, seeds=[SEED_GAME], bump=game.bump,
              constraint = game.authority == authority.key() @ GameError::NotAuthority)]
    pub game: Account<'info, Game>,
}



