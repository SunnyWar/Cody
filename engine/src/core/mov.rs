// src/core/mov.rs

use std::fmt;

use bitboard::{Square, piece::PieceKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Move {
    pub from: Square, // 0..63
    pub to: Square,   // 0..63
    pub promotion: Option<PieceKind>,
    pub flags: u8, // bit flags for special moves
}

impl Move {
    pub fn new(from: Square, to: Square) -> Self {
        Move {
            from,
            to,
            promotion: None,
            flags: 0,
        }
    }

    pub fn null() -> Self {
        Move {
            from: Square::A1,
            to: Square::A1,
            promotion: None,
            flags: 0,
        }
    }

    pub(crate) fn from(&self) -> Square {
        Square::A1
    }

    pub(crate) fn to(&self) -> Square {
        Square::A1
    }

    pub(crate) fn promotion(&self) -> Option<PieceKind> {
        None
    }
}

impl Move {
    fn from_square(&self) -> String {
        square_to_string(self.from)
    }

    fn to_square(&self) -> String {
        square_to_string(self.to)
    }
}

fn square_to_string(sq: Square) -> String {
    format!("{}{}", sq.file_char(), sq.rank_char())
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.from_square(), self.to_square())
    }
}
