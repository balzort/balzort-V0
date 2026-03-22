use anchor_lang::prelude::*;

#[derive(InitSpace)]
#[account]
pub struct Game {
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub treasury_fee_bps: u16,
    pub is_paused: bool,
    pub tournament_count: u64,
    pub bump: u8,
}

impl Game {
    pub const SPACE: usize = 8 + 32 * 2 + 2 + 1 + 8 + 1;
}


