use anchor_lang::prelude::*;

#[account]
pub struct TournamentEntry {
    pub tournament:        Pubkey,
    pub player:            Pubkey,
    pub entry_deposit:     u64,
    pub puzzle_nonce:      u64,
    pub parimutuel_weight: u128,
    pub completed:         bool,
    pub has_claimed:       bool,
    pub bump:              u8,
}

impl TournamentEntry {
    pub const SPACE: usize = 8    // discriminator
        + 32  // tournament
        + 32  // player
        + 8   // entry_deposit
        + 8   // puzzle_nonce
        + 16  // parimutuel_weight
        + 1   // completed
        + 1   // has_claimed
        + 1;  // bump
}