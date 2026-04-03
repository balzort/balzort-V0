use anchor_lang::prelude::*;
use crate::types::constants::*;

#[account]
pub struct Tournament {
    pub authority:         Pubkey,
    pub token_mint:        Pubkey,   // USDC mint specified at creation
    pub entry_fee:         u64,
    pub prize_pool:        u64,
    pub net_prize_pool:    u64,
    pub treasury_fee_bps:  u16,
    pub difficulty:        u8,
    pub num_tubes:         u8,       // puzzle config locked at creation
    pub balls_per_tube:    u8,       // puzzle config locked at creation
    pub max_time_secs:     u64,      // qualifying time window in seconds
    pub start_time:        i64,
    pub end_time:          i64,
    pub total_entries:     u32,
    pub total_completers:  u32,
    pub cumulative_weight: u128,
    pub is_closed:         bool,
    pub tournament_id:     u64,
    pub bump:              u8,
}

impl Tournament {
    pub const SPACE: usize = 8    // discriminator
        + 32  // authority
        + 32  // token_mint
        + 8   // entry_fee
        + 8   // prize_pool
        + 8   // net_prize_pool
        + 2   // treasury_fee_bps
        + 1   // difficulty
        + 1   // num_tubes
        + 1   // balls_per_tube
        + 8   // max_time_secs
        + 8   // start_time
        + 8   // end_time
        + 4   // total_entries
        + 4   // total_completers
        + 16  // cumulative_weight
        + 1   // is_closed
        + 8   // tournament_id
        + 1;  // bump

    pub fn is_open(&self, now: i64) -> bool {
        !self.is_closed && now < self.end_time
    }

    pub fn calculate_fee(prize_pool: u64, fee_bps: u16) -> (u64, u64) {
        let fee = (prize_pool as u128)
            .saturating_mul(fee_bps as u128)
            .checked_div(BPS_DENOMINATOR as u128)
            .unwrap_or(0) as u64;
        (fee, prize_pool.saturating_sub(fee))
    }
}