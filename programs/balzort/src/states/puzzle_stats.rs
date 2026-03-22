use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct PuzzleStats {
    pub status: u8,
    pub difficulty: u8,
    pub num_tubes: u8,
    pub balls_per_tube: u8,
    pub move_count: u32,
    pub undo_count: u32,
    pub started_at: i64,
    pub completed_at: i64,
    pub is_solved: bool,
    pub final_score: u64,
}

