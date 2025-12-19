use serde::{Deserialize, Serialize};

pub const COLOR_COUNT: usize = 9;

/// 颜色编码（与 `rules.json` 保持一致）。
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Color {
    Red = 0,
    Blue = 1,
    Black = 2,
    Green = 3,
    Yellow = 4,
    Purple = 5,
    White = 6,
    Orange = 7,
    Cyan = 8,
}

impl Color {
    pub fn from_u8(v: u8) -> Option<Self> {
        Some(match v {
            0 => Self::Red,
            1 => Self::Blue,
            2 => Self::Black,
            3 => Self::Green,
            4 => Self::Yellow,
            5 => Self::Purple,
            6 => Self::White,
            7 => Self::Orange,
            8 => Self::Cyan,
            _ => return None,
        })
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

pub const NON_WHITE_COLORS: [Color; 8] = [
    Color::Red,
    Color::Blue,
    Color::Black,
    Color::Green,
    Color::Yellow,
    Color::Purple,
    Color::Orange,
    Color::Cyan,
];
