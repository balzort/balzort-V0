use crate::states::{Game, Player, PuzzleStats};
use crate::types::constants::*;
use crate::types::{GameError, PuzzleAbandoned, PuzzleStatus};
use anchor_lang::prelude::*;

pub fn handle_abandon_puzzle(ctx: Context<AbandonPuzzle>) -> Result<()> {
    let puzzle_stats_key = ctx.accounts.puzzle_stats.key();
    let stats = &mut ctx.accounts.puzzle_stats;

    stats.status = PuzzleStatus::Abandoned as u8;

     emit!(PuzzleAbandoned {
        player: ctx.accounts.player.wallet,
        puzzle_board: puzzle_stats_key,
        puzzle_stats: puzzle_stats_key,
        move_count: stats.move_count,
        undo_count: stats.undo_count,
        difficulty: stats.difficulty,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct AbandonPuzzle<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(seeds=[SEED_PLAYER, player.wallet.as_ref()], bump=player.bump,
              constraint = player.wallet == signer.key()
                        || player.session_key == Some(signer.key()) @ GameError::Unauthorized)]
    pub player: Account<'info, Player>,

    #[account(seeds=[SEED_GAME], bump=game.bump)]
    pub game: Account<'info, Game>,

    #[account(
        mut,
        seeds = [SEED_PUZZLE_STATS, player.key().as_ref(), &player.puzzles_started_nonce.saturating_sub(1).to_le_bytes()],
        bump
    )]
    pub puzzle_stats: Account<'info, PuzzleStats>,
}




