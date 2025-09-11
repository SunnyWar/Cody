// src/core/piece.rs

use crate::core::square::Square;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PieceKind {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    #[inline]
    pub fn opposite(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Piece {
    WhitePawn = 0,
    WhiteKnight = 1,
    WhiteBishop = 2,
    WhiteRook = 3,
    WhiteQueen = 4,
    WhiteKing = 5,
    BlackPawn = 6,
    BlackKnight = 7,
    BlackBishop = 8,
    BlackRook = 9,
    BlackQueen = 10,
    BlackKing = 11,
    None = 12,
}

impl Piece {
    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    #[inline]
    pub const fn from_parts(color: Color, kind: Option<PieceKind>) -> Self {
        match kind {
            Some(k) => {
                let base = color as u8 * 6;
                Self::from_index(base + k as u8)
            }
            None => Self::None,
        }
    }

    const fn from_index(idx: u8) -> Self {
        match idx {
            0 => Self::WhitePawn,
            1 => Self::WhiteKnight,
            2 => Self::WhiteBishop,
            3 => Self::WhiteRook,
            4 => Self::WhiteQueen,
            5 => Self::WhiteKing,
            6 => Self::BlackPawn,
            7 => Self::BlackKnight,
            8 => Self::BlackBishop,
            9 => Self::BlackRook,
            10 => Self::BlackQueen,
            11 => Self::BlackKing,
            12 => Self::None,
            _ => panic!("Invalid piece index"),
        }
    }

    #[inline]
    const fn piece_index(color: Color, kind: PieceKind) -> usize {
        match (color, kind) {
            (Color::White, PieceKind::Pawn) => 0,
            (Color::White, PieceKind::Knight) => 1,
            (Color::White, PieceKind::Bishop) => 2,
            (Color::White, PieceKind::Rook) => 3,
            (Color::White, PieceKind::Queen) => 4,
            (Color::White, PieceKind::King) => 5,
            (Color::Black, PieceKind::Pawn) => 6,
            (Color::Black, PieceKind::Knight) => 7,
            (Color::Black, PieceKind::Bishop) => 8,
            (Color::Black, PieceKind::Rook) => 9,
            (Color::Black, PieceKind::Queen) => 10,
            (Color::Black, PieceKind::King) => 11,
        }
    }

    #[inline]
    pub const fn color(self) -> Color {
        if (self as u8) < 6 {
            Color::White
        } else {
            Color::Black
        }
    }

    #[inline]
    pub const fn kind(self) -> PieceKind {
        unsafe { std::mem::transmute((self as u8) % 6) }
    }
}

impl Piece {
    #[inline]
    pub const fn to_char(self) -> char {
        use Piece::*;
        match self {
            WhitePawn => 'P',
            WhiteKnight => 'N',
            WhiteBishop => 'B',
            WhiteRook => 'R',
            WhiteQueen => 'Q',
            WhiteKing => 'K',
            BlackPawn => 'p',
            BlackKnight => 'n',
            BlackBishop => 'b',
            BlackRook => 'r',
            BlackQueen => 'q',
            BlackKing => 'k',
            None => ' ', // TODO - is this an error?
        }
    }
}

impl Piece {
    pub fn from_char(c: char) -> Option<Self> {
        use Piece::*;
        match c {
            'P' => Some(WhitePawn),
            'N' => Some(WhiteKnight),
            'B' => Some(WhiteBishop),
            'R' => Some(WhiteRook),
            'Q' => Some(WhiteQueen),
            'K' => Some(WhiteKing),
            'p' => Some(BlackPawn),
            'n' => Some(BlackKnight),
            'b' => Some(BlackBishop),
            'r' => Some(BlackRook),
            'q' => Some(BlackQueen),
            'k' => Some(BlackKing),
            _ => Some(None),
        }
    }
}

impl PieceKind {
    /// Parse from a promotion character in UCI notation ('q','r','b','n')
    pub fn from_uci(ch: char) -> Option<Self> {
        match ch {
            'q' => Some(PieceKind::Queen),
            'r' => Some(PieceKind::Rook),
            'b' => Some(PieceKind::Bishop),
            'n' => Some(PieceKind::Knight),
            _ => None,
        }
    }
}


