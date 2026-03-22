use crate::types::constants::PARIMUTUEL_SCALAR;

pub fn parimutuel_weight(elapsed_secs: u64, move_count: u32) -> u128 {
    let time  = elapsed_secs.max(1) as u128;
    let moves = (move_count as u128).max(1);
    PARIMUTUEL_SCALAR.checked_div(time.saturating_mul(moves)).unwrap_or(0)
}

