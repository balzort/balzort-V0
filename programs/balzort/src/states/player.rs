use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Player {
    pub wallet: Pubkey,
    pub session_key: Option<Pubkey>,
    pub session_expires_at: i64,
    pub total_puzzles_solved: u64,
    pub puzzles_started_nonce: u64,
    pub vrf_randomness: [u8; 32],
    pub bump: u8,
}

impl Player {
    pub const SPACE: usize = 8      
        + 32 
        + 1+32 
        + 8   
        + 8   
        + 8   
        + 32  
        + 1; 

    pub fn is_valid_signer(&self, signer: &Pubkey, now: i64) -> bool {
        if *signer == self.wallet {
            return true;
        }
        if let Some(sk) = self.session_key {
            if *signer == sk && now < self.session_expires_at {
                return true;
            }
        }
        false
    }
}


