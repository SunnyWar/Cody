// src/core/castling.rs

use bitboard::piece::Color;

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

    pub fn clear(&mut self, color: Color, side: bool) {
        match (color, side) {
            (Color::White, true) => self.white_kingside = false,
            (Color::White, false) => self.white_queenside = false,
            (Color::Black, true) => self.black_kingside = false,
            (Color::Black, false) => self.black_queenside = false,
        }
    }
}
