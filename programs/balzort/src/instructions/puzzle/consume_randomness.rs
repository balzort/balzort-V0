use crate::states::{Player, PuzzleStats};
use crate::types::constants::*;
use crate::types::PuzzleStatus;
use anchor_lang::prelude::*;

pub fn handle_consume_randomness(
    ctx: Context<ConsumeRandomness>,
    randomness: [u8; 32],
) -> Result<()> {
    let auth = &mut ctx.accounts.player;
    auth.vrf_randomness = randomness;

    let stats = &mut ctx.accounts.puzzle_stats;
    stats.status = PuzzleStatus::BoardReady as u8;

    Ok(())
}

#[derive(Accounts)]
pub struct ConsumeRandomness<'info> {
    #[account(address = ephemeral_vrf_sdk::consts::VRF_PROGRAM_IDENTITY)]
    pub vrf_program_identity: Signer<'info>,

    #[account(mut)]
    pub player: Account<'info, Player>,

    #[account(
        mut,
        seeds = [SEED_PUZZLE_STATS, player.key().as_ref(), &player.puzzles_started_nonce.saturating_sub(1).to_le_bytes()],
        bump
    )]
    pub puzzle_stats: Account<'info, PuzzleStats>,
}




