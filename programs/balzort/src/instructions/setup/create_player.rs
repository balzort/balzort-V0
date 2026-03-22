use crate::states::{Game, Player};
use crate::types::constants::*;
use anchor_lang::prelude::*;

pub fn handle_create_player(ctx: Context<CreatePlayer>) -> Result<()> {
    let auth = &mut ctx.accounts.player;
    let player_key = ctx.accounts.signer.key();

    auth.wallet = player_key;
    auth.session_key = None;
    auth.session_expires_at = 0;
    auth.total_puzzles_solved = 0;
    auth.puzzles_started_nonce = 0;
    auth.bump = ctx.bumps.player;

    Ok(())
}

#[derive(Accounts)]
pub struct CreatePlayer<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub signer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = Player::SPACE,
        seeds = [SEED_PLAYER, signer.key().as_ref()],
        bump
    )]
    pub player: Account<'info, Player>,

    #[account(seeds = [SEED_GAME], bump = game.bump)]
    pub game: Account<'info, Game>,

    pub system_program: Program<'info, System>,
}