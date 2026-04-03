pub const SEED_GAME: &[u8] = b"game";
pub const SEED_PLAYER: &[u8] = b"player";
pub const SEED_TOURNAMENT: &[u8] = b"tournament";
pub const SEED_TOURNAMENT_ENTRY: &[u8] = b"tournament_entry";
pub const SEED_TOURNAMENT_VAULT: &[u8] = b"tournament_vault"; 
pub const SEED_PUZZLE_BOARD: &[u8] = b"puzzle_board";
pub const SEED_PUZZLE_STATS: &[u8] = b"puzzle_stats";

pub const MAX_TREASURY_FEE_BPS: u16 = 2_000;
pub const BPS_DENOMINATOR: u64 = 10_000;

pub const MIN_SESSION_DURATION_SECS: i64 = 60;
pub const MAX_SESSION_DURATION_SECS: i64 = 3_600;

pub const PARIMUTUEL_SCALAR: u128 = 1_000_000_000_000;