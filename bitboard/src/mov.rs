// bitboard/mov.rs

use crate::Square;
use crate::piece::PieceKind;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoveType {
    Quiet,
    Capture,
    Promotion(PieceKind),
    EnPassant,
    CastleKingside,
    CastleQueenside,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChessMove {
    pub from: Square,
    pub to: Square,
    pub move_type: MoveType,
}

impl ChessMove {
    pub fn new(from: Square, to: Square, move_type: MoveType) -> Self {
        ChessMove {
            from,
            to,
            move_type,
        }
    }

    pub fn null() -> Self {
        ChessMove {
            from: Square::A1,
            to: Square::A1,
            move_type: MoveType::Quiet, // or a dedicated Null variant if you prefer
        }
    }

    pub fn is_null(&self) -> bool {
        self.from == Square::A1 && self.to == Square::A1
    }

    pub fn from(&self) -> Square {
        self.from
    }

    pub fn to(&self) -> Square {
        self.to
    }

    pub fn promotion(&self) -> Option<PieceKind> {
        match self.move_type {
            MoveType::Promotion(kind) => Some(kind),
            _ => None,
        }
    }

    pub fn from_square(&self) -> String {
        square_to_string(self.from)
    }

    pub fn to_square(&self) -> String {
        square_to_string(self.to)
    }
}

fn square_to_string(sq: Square) -> String {
    format!("{}{}", sq.file_char(), sq.rank_char())
}

impl fmt::Display for ChessMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.move_type {
            MoveType::Promotion(kind) => {
                write!(
                    f,
                    "{}{}{}",
                    self.from_square(),
                    self.to_square(),
                    kind.to_uci()
                )
            }
            _ => write!(f, "{}{}", self.from_square(), self.to_square()),
        }
    }
}
