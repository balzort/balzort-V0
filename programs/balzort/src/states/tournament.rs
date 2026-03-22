use anchor_lang::prelude::*;
use crate::types::constants::*;

#[account]
pub struct Tournament {
    pub authority:         Pubkey,
    pub entry_fee:         u64,
    pub prize_pool:        u64,
    pub net_prize_pool:    u64,    
    pub treasury_fee_bps:  u16,   
    pub difficulty:        u8,    
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
    pub const SPACE: usize = 8 + 32 + 8+8+8 + 2 + 1 + 8+8 + 4+4 + 16 + 1 + 8 + 1;

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

