// src/core/square.rs

use crate::core::{
    bitboard::{FILE_MASKS, RANK_MASKS},
    bitboardmask::BitBoardMask,
};

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[rustfmt::skip]
pub enum Square {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}

impl Square {
    /// Create a Square from file ('a'..'h') and rank ('1'..'8') chars.
    pub fn from_coords(file: char, rank: char) -> Option<Self> {
        if !('a'..='h').contains(&file) || !('1'..='8').contains(&rank) {
            return None;
        }
        let file_idx = (file as u8) - b'a';
        let rank_idx = (rank as u8) - b'1';
        let idx = rank_idx * 8 + file_idx;
        // Safe because idx is guaranteed 0..63
        Some(unsafe { std::mem::transmute::<u8, Square>(idx) })
    }

    pub const fn file(self) -> u8 {
        (self as u8) % 8
    }

    pub const fn rank(self) -> u8 {
        (self as u8) / 8
    }

    pub const fn bit(self) -> BitBoardMask {
        BitBoardMask::from_square(self)
    }

    pub fn to_uci(self) -> String {
        let file_char = (b'a' + self.file()) as char;
        let rank_char = (b'1' + self.rank()) as char;
        format!("{}{}", file_char, rank_char)
    }

    #[inline]
    pub const fn idx(self) -> usize {
        self as u8 as usize
    }

    pub const fn from_index(idx: u8) -> Self {
        unsafe { std::mem::transmute::<u8, Square>(idx) }
    }

    #[rustfmt::skip]
    pub fn from_rank_file(rank: u8, file: u8) -> Option<Self> {
        use Square::*;
        match (rank, file) {
            (0, 0) => Some(A1), (0, 1) => Some(B1), (0, 2) => Some(C1), (0, 3) => Some(D1),
            (0, 4) => Some(E1), (0, 5) => Some(F1), (0, 6) => Some(G1), (0, 7) => Some(H1),
            (1, 0) => Some(A2), (1, 1) => Some(B2), (1, 2) => Some(C2), (1, 3) => Some(D2),
            (1, 4) => Some(E2), (1, 5) => Some(F2), (1, 6) => Some(G2), (1, 7) => Some(H2),
            (2, 0) => Some(A3), (2, 1) => Some(B3), (2, 2) => Some(C3), (2, 3) => Some(D3),
            (2, 4) => Some(E3), (2, 5) => Some(F3), (2, 6) => Some(G3), (2, 7) => Some(H3),
            (3, 0) => Some(A4), (3, 1) => Some(B4), (3, 2) => Some(C4), (3, 3) => Some(D4),
            (3, 4) => Some(E4), (3, 5) => Some(F4), (3, 6) => Some(G4), (3, 7) => Some(H4),
            (4, 0) => Some(A5), (4, 1) => Some(B5), (4, 2) => Some(C5), (4, 3) => Some(D5),
            (4, 4) => Some(E5), (4, 5) => Some(F5), (4, 6) => Some(G5), (4, 7) => Some(H5),
            (5, 0) => Some(A6), (5, 1) => Some(B6), (5, 2) => Some(C6), (5, 3) => Some(D6),
            (5, 4) => Some(E6), (5, 5) => Some(F6), (5, 6) => Some(G6), (5, 7) => Some(H6),
            (6, 0) => Some(A7), (6, 1) => Some(B7), (6, 2) => Some(C7), (6, 3) => Some(D7),
            (6, 4) => Some(E7), (6, 5) => Some(F7), (6, 6) => Some(G7), (6, 7) => Some(H7),
            (7, 0) => Some(A8), (7, 1) => Some(B8), (7, 2) => Some(C8), (7, 3) => Some(D8),
            (7, 4) => Some(E8), (7, 5) => Some(F8), (7, 6) => Some(G8), (7, 7) => Some(H8),
            _ => None,
        }
    }

    fn iter_all() -> impl Iterator<Item = Square> {
        (0u8..64).map(|v| unsafe { std::mem::transmute::<u8, Square>(v) })
    }

    pub const fn all_array() -> [Square; 64] {
        let mut squares = [Square::A1; 64];
        let mut i = 0;
        while i < 64 {
            squares[i] = unsafe { std::mem::transmute::<u8, Square>(i as u8) };
            i += 1;
        }
        squares
    }

    pub fn file_char(self) -> char {
        (b'a' + self.file()) as char
    }

    pub fn rank_char(self) -> char {
        (b'1' + self.rank()) as char
    }

    pub fn to_string(self) -> String {
        format!("{}{}", self.file_char(), self.rank_char())
    }

    pub fn forward(self, n: u8) -> Option<Self> {
        let rank = self.rank();
        let file = self.file();
        let new_rank = rank.checked_add(n)?;
        Square::from_rank_file(new_rank, file)
    }

    pub fn backward(self, n: u8) -> Option<Self> {
        let rank = self.rank();
        let file = self.file();
        let new_rank = rank.checked_sub(n)?;
        Square::from_rank_file(new_rank, file)
    }

    pub fn advance(self, offset: i8) -> Option<Self> {
        let idx = self as i8 + offset;
        if idx >= 0 && idx < 64 {
            Some(unsafe { std::mem::transmute::<u8, Square>(idx as u8) })
        } else {
            None
        }
    }

    pub const fn file_mask(self) -> BitBoardMask {
        FILE_MASKS[self.file() as usize]
    }

    pub const fn rank_mask(self) -> BitBoardMask {
        RANK_MASKS[self.rank() as usize]
    }
}
