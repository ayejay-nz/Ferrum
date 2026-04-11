use std::fmt;

use crate::position::Position;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum GameResult {
    BlackWin,
    Draw,
    WhiteWin,
}

impl GameResult {
    pub fn from_pgn_tag(tag: &str) -> Option<Self> {
        match tag {
            "0-1" => Some(Self::BlackWin),
            "1-0" => Some(Self::WhiteWin),
            "1/2-1/2" => Some(Self::Draw),
            "*" => None,
            _ => None,
        }
    }

    pub fn to_value(self) -> f64 {
        match self {
            Self::BlackWin => 0f64,
            Self::WhiteWin => 1f64,
            Self::Draw => 0.5,
        }
    }
}

impl fmt::Display for GameResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlackWin => f.write_str("0-1"),
            Self::WhiteWin => f.write_str("1-0"),
            Self::Draw => f.write_str("1/2-1/2"),
        }
    }
}

#[derive(Copy, Clone)]
pub struct Sample {
    pub pos: Position,
    pub result: GameResult,
}
