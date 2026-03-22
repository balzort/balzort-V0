use crate::states::{Game, Player, PuzzleBoard, PuzzleStats};
use crate::types::constants::*;
use crate::types::GameError;
use crate::utils::validate_signer;
use anchor_lang::prelude::*;

pub fn handle_apply_undo(ctx: Context<ApplyUndo>) -> Result<()> {
    let clock = Clock::get()?;
    validate_signer(
        &ctx.accounts.signer.key(),
        &*ctx.accounts.player,
        clock.unix_timestamp,
    )?;

    let board = &mut ctx.accounts.puzzle_board;
    let stats = &mut ctx.accounts.puzzle_stats;

    require!(stats.status == 2, GameError::InvalidState);
    require!(board.has_undo, GameError::InvalidState);

    let from = board.undo_from as usize;
    let to = board.undo_to as usize;
    let ball = board.undo_ball;

    let to_top =
        to * crate::states::puzzle_board::MAX_CAPACITY + (board.tube_lengths[to] as usize - 1);
    board.balls[to_top] = 0;
    board.tube_lengths[to] -= 1;

    let from_top =
        from * crate::states::puzzle_board::MAX_CAPACITY + board.tube_lengths[from] as usize;
    board.balls[from_top] = ball;
    board.tube_lengths[from] += 1;

    board.has_undo = false;
    board.undo_from = 0;
    board.undo_to = 0;
    board.undo_ball = 0;

    stats.move_count = stats.move_count.saturating_add(1);
    stats.undo_count = stats.undo_count.saturating_add(1);

    Ok(())
}

#[derive(Accounts)]
pub struct ApplyUndo<'info> {
    pub signer: Signer<'info>,

    #[account(seeds=[SEED_PLAYER, player.wallet.as_ref()], bump=player.bump,
              constraint = player.wallet == signer.key()
                        || player.session_key == Some(signer.key()) @ GameError::Unauthorized)]
    pub player: Account<'info, Player>,

    #[account(seeds=[SEED_GAME], bump=game.bump)]
    pub game: Account<'info, Game>,

    #[account(
        mut,
        seeds = [SEED_PUZZLE_BOARD, player.key().as_ref(), &player.puzzles_started_nonce.saturating_sub(1).to_le_bytes()],
        bump
    )]
    pub puzzle_board: Account<'info, PuzzleBoard>,

    #[account(
        mut,
        seeds = [SEED_PUZZLE_STATS, player.key().as_ref(), &player.puzzles_started_nonce.saturating_sub(1).to_le_bytes()],
        bump
    )]
    pub puzzle_stats: Account<'info, PuzzleStats>,
}



