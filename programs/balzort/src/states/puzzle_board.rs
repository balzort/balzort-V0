use anchor_lang::prelude::*;

pub const MAX_TUBES: usize = 12;
pub const MAX_CAPACITY: usize = 10;
pub const BALLS_LEN: usize = MAX_TUBES * MAX_CAPACITY;

#[account]
#[derive(InitSpace)]
pub struct PuzzleBoard {
    pub num_tubes: u8,
    pub num_colors: u8,
    pub max_capacity: u8,
    pub balls: [u8; BALLS_LEN],
    pub tube_lengths: [u8; MAX_TUBES],
    pub vrf_seed: [u8; 32],
    pub undo_from: u8,
    pub undo_to: u8,
    pub undo_ball: u8,
    pub has_undo: bool,
}

