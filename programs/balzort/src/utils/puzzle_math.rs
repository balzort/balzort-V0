use crate::states::PuzzleBoard;

pub fn is_solved(board: &PuzzleBoard) -> bool {
    let max_capacity = crate::states::puzzle_board::MAX_CAPACITY;
    for i in 0..board.num_tubes as usize {
        let len = board.tube_lengths[i] as usize;
        if len == 0 {
            continue;
        }

        let base = i * max_capacity;
        let first_color = board.balls[base];
        if first_color == 0 {
            return false;
        }

        for j in 1..len {
            if board.balls[base + j] != first_color {
                return false;
            }
        }
    }
    true
}

pub fn compute_score(
    difficulty: u8,
    move_count: u32,
    elapsed_secs: u64,
    undo_count: u32,
    num_colors: u8,
    max_capacity: u8,
) -> u64 {
    const SPEED_WINDOW: u64 = 300;
    const UNDO_PENALTY: u64 = 50;
    const BASE_POINTS: u64 = 10_000;

    let multiplier: u64 = match difficulty {
        0 => 1,
        1 => 2,
        _ => 3,
    };
    let base = BASE_POINTS.saturating_mul(multiplier);
    let optimal = ((num_colors as u64) * (max_capacity as u64)) / 2;
    let actual = (move_count as u64).max(1);

    let efficiency = if actual <= optimal {
        base
    } else {
        base.saturating_mul(optimal)
            .checked_div(actual)
            .unwrap_or(0)
    };

    let speed_bonus = if elapsed_secs >= SPEED_WINDOW {
        0
    } else {
        base.saturating_mul(SPEED_WINDOW - elapsed_secs)
            .checked_div(SPEED_WINDOW)
            .unwrap_or(0)
    };

    let penalty = UNDO_PENALTY.saturating_mul(undo_count as u64);
    efficiency
        .saturating_add(speed_bonus)
        .saturating_sub(penalty)
}

pub struct SeededRng {
    pub state: [u8; 32],
    pub pos: usize,
}

impl SeededRng {
    pub fn new(seed: [u8; 32]) -> Self {
        Self {
            state: seed,
            pos: 0,
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        if self.pos + 8 > 32 {
            for i in 0..32 {
                self.state[i] ^= self.state[(i + 7) % 32]
                    .wrapping_add(self.state[(i + 13) % 32])
                    .rotate_left(3);
            }
            self.pos = 0;
        }
        let bytes: [u8; 8] = self.state[self.pos..self.pos + 8].try_into().unwrap();
        self.pos += 8;
        u64::from_le_bytes(bytes)
    }

    pub fn next_bounded(&mut self, bound: u64) -> u64 {
        if bound == 0 {
            return 0;
        }
        self.next_u64() % bound
    }
}

