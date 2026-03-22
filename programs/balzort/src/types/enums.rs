use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum PuzzleStatus {
    Initialized = 0,
    BoardReady = 1,
    Started = 2,
    Solved = 3,
    Finalized = 4,
    Abandoned = 5,
}

impl PuzzleStatus {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0 => Some(Self::Initialized),
            1 => Some(Self::BoardReady),
            2 => Some(Self::Started),
            3 => Some(Self::Solved),
            4 => Some(Self::Finalized),
            5 => Some(Self::Abandoned),
            _ => None,
        }
    }
}

impl Default for PuzzleStatus {
    fn default() -> Self {
        Self::Initialized
    }
}

