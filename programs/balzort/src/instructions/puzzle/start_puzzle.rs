use crate::states::{Game, Player, PuzzleBoard, PuzzleStats};
use crate::types::constants::*;
use crate::types::{GameError, PuzzleStarted, PuzzleStatus};
use crate::utils::{validate_signer, SeededRng};
use anchor_lang::prelude::*;

pub fn handle_start_puzzle(ctx: Context<StartPuzzle>) -> Result<()> {
    let clock = Clock::get()?;
    validate_signer(
        &ctx.accounts.signer.key(),
        &*ctx.accounts.player,
        clock.unix_timestamp,
    )?;

    let stats = &mut ctx.accounts.puzzle_stats;

    require!(
        stats.status == PuzzleStatus::BoardReady as u8, GameError::InvalidState
    );

    let board = &mut ctx.accounts.puzzle_board;

    let num_tubes = stats.num_tubes;
    let balls_per_tube = stats.balls_per_tube;
    let difficulty = stats.difficulty;
    let vrf_seed = ctx.accounts.player.vrf_randomness;
    let num_colors = num_tubes - 1;

    let mut balls = [0u8; crate::states::puzzle_board::BALLS_LEN];
    let mut tube_lengths = [0u8; crate::states::puzzle_board::MAX_TUBES];

    for color in 0..num_colors as usize {
        for slot in 0..balls_per_tube as usize {
            balls[color * crate::states::puzzle_board::MAX_CAPACITY as usize + slot] =
                (color + 1) as u8;
        }
        tube_lengths[color] = balls_per_tube;
    }

    let filled = num_colors as usize * balls_per_tube as usize;
    let mut rng = SeededRng::new(vrf_seed);

    let get_real_index = |idx: usize| -> usize {
        let bpt = balls_per_tube as usize;
        let max_cap = crate::states::puzzle_board::MAX_CAPACITY as usize;
        (idx / bpt) * max_cap + (idx % bpt)
    };

    match difficulty {
        0 => {
            for i in (1..filled).rev() {
                let window = 5.min(i as u64);
                let j = i - rng.next_bounded(window + 1) as usize;
                balls.swap(get_real_index(i), get_real_index(j));
            }
        }
        1 => {
            for i in (1..filled).rev() {
                let j = rng.next_bounded(i as u64 + 1) as usize;
                balls.swap(get_real_index(i), get_real_index(j));
            }
        }
        __ => {
            for i in (1..filled).rev() {
                let j = rng.next_bounded(i as u64 + 1) as usize;
                balls.swap(get_real_index(i), get_real_index(j));
            }

            let bpt = balls_per_tube as usize;
            for _ in 0..3 {
                for i in 1..filled {
                    if i % bpt != 0 {
                        let curr = get_real_index(i);
                        let prev = get_real_index(i - 1);

                        if balls[curr] == balls[prev] {
                            for _attempt in 0..7 {
                                let j = rng.next_bounded(filled as u64) as usize;
                                let target = get_real_index(j);
                                if balls[target] != balls[curr] {
                                    balls.swap(curr, target);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    board.num_tubes = num_tubes;
    board.num_colors = num_colors;
    board.max_capacity = balls_per_tube;
    board.balls = balls;
    board.tube_lengths = tube_lengths;
    board.vrf_seed = vrf_seed;
    board.has_undo = false;
    board.undo_from = 0;
    board.undo_to = 0;
    board.undo_ball = 0;

    stats.status = PuzzleStatus::Started as u8;
    stats.move_count = 0;
    stats.undo_count = 0;
    stats.started_at = clock.unix_timestamp;
    stats.completed_at = 0;
    stats.is_solved = false;
    stats.final_score = 0;

     emit!(PuzzleStarted {
        puzzle_board: ctx.accounts.puzzle_board.key(),
        player: ctx.accounts.player.wallet,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct StartPuzzle<'info> {
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




