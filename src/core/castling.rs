// src/core/castling.rs
use crate::core::{bitboardmask::BitBoardMask, piece::Color, square::Square};

#[derive(Clone, Copy)]
pub struct CastlingRights {
    pub(crate) white_kingside: bool,
    pub(crate) white_queenside: bool,
    pub(crate) black_kingside: bool,
    pub(crate) black_queenside: bool,
}

impl CastlingRights {
    pub fn empty() -> Self {
        Self {
            white_kingside: false,
            white_queenside: false,
            black_kingside: false,
            black_queenside: false,
        }
    }

    pub fn from_fen(s: &str) -> Self {
        let mut rights = Self::empty();
        if s.contains('K') {
            rights.white_kingside = true;
        }
        if s.contains('Q') {
            rights.white_queenside = true;
        }
        if s.contains('k') {
            rights.black_kingside = true;
        }
        if s.contains('q') {
            rights.black_queenside = true;
        }
        rights
    }

    pub fn to_fen(&self) -> String {
        let mut s = String::new();
        if self.white_kingside {
            s.push('K');
        }
        if self.white_queenside {
            s.push('Q');
        }
        if self.black_kingside {
            s.push('k');
        }
        if self.black_queenside {
            s.push('q');
        }
        if s.is_empty() {
            s.push('-');
        }
        s
    }

    pub fn to_bits(&self) -> u8 {
        (self.white_kingside as u8)
            | ((self.white_queenside as u8) << 1)
            | ((self.black_kingside as u8) << 2)
            | ((self.black_queenside as u8) << 3)
    }

    pub fn clear(&mut self, color: Color, side: bool) {
        match (color, side) {
            (Color::White, true) => self.white_kingside = false,
            (Color::White, false) => self.white_queenside = false,
            (Color::Black, true) => self.black_kingside = false,
            (Color::Black, false) => self.black_queenside = false,
        }
    }
}

pub struct CastlingMeta {
    pub king_from: Square,
    pub kingside_to: Square,
    pub queenside_to: Square,
    pub kingside_mask: BitBoardMask,
    pub queenside_mask: BitBoardMask,
}

pub const WHITE_CASTLING: CastlingMeta = CastlingMeta {
    king_from: Square::E1,
    kingside_to: Square::G1,
    queenside_to: Square::C1,
    kingside_mask: BitBoardMask::from_squares(&[Square::F1, Square::G1]),
    queenside_mask: BitBoardMask::from_squares(&[Square::B1, Square::C1, Square::D1]),
};

pub const BLACK_CASTLING: CastlingMeta = CastlingMeta {
    king_from: Square::E8,
    kingside_to: Square::G8,
    queenside_to: Square::C8,
    kingside_mask: BitBoardMask::from_squares(&[Square::F8, Square::G8]),
    queenside_mask: BitBoardMask::from_squares(&[Square::B8, Square::C8, Square::D8]),
};
