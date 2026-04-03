use crate::states::{Game, Player, PuzzleBoard, PuzzleStats};
use crate::types::constants::*;
use crate::types::GameError;
use crate::utils::{compute_score, is_solved, validate_signer};
use anchor_lang::prelude::*;

pub fn handle_apply_move(ctx: Context<ApplyMove>, from_tube: u8, to_tube: u8) -> Result<()> {
    let clock = Clock::get()?;
    validate_signer(
        &ctx.accounts.signer.key(),
        &*ctx.accounts.player,
        clock.unix_timestamp,
    )?;

    let board = &mut ctx.accounts.puzzle_board;
    let stats = &mut ctx.accounts.puzzle_stats;

    require!(stats.status == 2, GameError::InvalidState);

    let from = from_tube as usize;
    let to = to_tube as usize;
    let from_top =
        from * crate::states::puzzle_board::MAX_CAPACITY + (board.tube_lengths[from] as usize - 1);
    let ball = board.balls[from_top];
    board.balls[from_top] = 0;
    board.tube_lengths[from] -= 1;

    let to_top = to * crate::states::puzzle_board::MAX_CAPACITY + board.tube_lengths[to] as usize;
    board.balls[to_top] = ball;
    board.tube_lengths[to] += 1;

    board.undo_from = from_tube;
    board.undo_to = to_tube;
    board.undo_ball = ball;
    board.has_undo = true;

    stats.move_count = stats.move_count.saturating_add(1);

    // Start the timer on the FIRST move — ensures both started_at and
    // completed_at use the TEE clock (avoids L1↔TEE clock drift).
    if stats.started_at == 0 {
        stats.started_at = clock.unix_timestamp;
    }

    if is_solved(board) {
        stats.is_solved = true;
        stats.completed_at = clock.unix_timestamp;
        stats.status = 3;

        let elapsed_secs = (stats.completed_at - stats.started_at).max(0) as u64;
        stats.final_score = compute_score(
            stats.difficulty,
            stats.move_count,
            elapsed_secs,
            stats.undo_count,
            board.num_colors,
            board.max_capacity,
        );
    }

    Ok(())
}

#[derive(Accounts)]
pub struct ApplyMove<'info> {
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




