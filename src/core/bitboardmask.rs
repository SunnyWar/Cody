// src/core/bitboardmask.rs
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};
use std::ops::{Shl, Shr};

use crate::core::{
    bitboard::{
        BISHOP_ATTACKS, BISHOP_MASKS, KING_ATTACKS, KNIGHT_ATTACKS, PAWN_ATTACKS, ROOK_ATTACKS,
        ROOK_MASKS, occ_to_index,
    },
    piece::Color,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitBoardMask(pub u64);

impl BitBoardMask {
    #[inline]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn from_square(sq: u8) -> Self {
        Self(1u64 << sq)
    }

    #[inline]
    pub fn set(&mut self, sq: u8) {
        self.0 |= 1u64 << sq;
    }

    #[inline]
    pub fn clear(&mut self, sq: u8) {
        self.0 &= !(1u64 << sq);
    }

    #[inline]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub const fn count(self) -> u32 {
        self.0.count_ones()
    }

    #[inline]
    pub fn is_empty_bb(bb: u64) -> bool {
        bb == 0
    }

    #[inline]
    pub fn is_nonempty(self) -> bool {
        self.0 != 0
    }

    #[inline]
    pub fn contains_square(self, sq: u8) -> bool {
        (self.0 >> sq) & 1 != 0
    }

    /// Returns an iterator over all set squares in this bitboard.
    #[inline]
    pub fn squares(self) -> impl Iterator<Item = u8> {
        let mut bb = self.0;
        std::iter::from_fn(move || {
            if bb == 0 {
                None
            } else {
                let sq = bb.trailing_zeros() as u8;
                bb &= bb - 1; // clear the lowest set bit
                Some(sq)
            }
        })
    }

    #[inline]
    pub fn first_square(self) -> Option<u8> {
        if self.0 == 0 {
            None
        } else {
            Some(self.0.trailing_zeros() as u8)
        }
    }
}

impl Shl<i8> for BitBoardMask {
    type Output = Self;
    fn shl(self, rhs: i8) -> Self::Output {
        if rhs >= 0 {
            BitBoardMask(self.0 << rhs)
        } else {
            BitBoardMask(self.0 >> -rhs)
        }
    }
}

impl Shr<i8> for BitBoardMask {
    type Output = Self;
    fn shr(self, rhs: i8) -> Self::Output {
        if rhs >= 0 {
            BitBoardMask(self.0 >> rhs)
        } else {
            BitBoardMask(self.0 << -rhs)
        }
    }
}

impl BitAnd for BitBoardMask {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        BitBoardMask(self.0 & rhs.0)
    }
}

impl BitAndAssign for BitBoardMask {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOr for BitBoardMask {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        BitBoardMask(self.0 | rhs.0)
    }
}

impl BitOrAssign for BitBoardMask {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl Not for BitBoardMask {
    type Output = Self;
    fn not(self) -> Self::Output {
        BitBoardMask(!self.0)
    }
}

#[inline]
pub fn rook_attacks_mask(from: u8, occ: BitBoardMask) -> BitBoardMask {
    let from = from as usize;
    let mask = ROOK_MASKS[from];
    let index = occ_to_index(occ.0 & mask, mask);
    BitBoardMask(ROOK_ATTACKS[from][index])
}

#[inline]
pub fn bishop_attacks_mask(from: u8, occ: BitBoardMask) -> BitBoardMask {
    let from = from as usize;
    let mask = BISHOP_MASKS[from];
    let index = occ_to_index(occ.0 & mask, mask);
    BitBoardMask(BISHOP_ATTACKS[from][index])
}

#[inline]
pub fn queen_attacks_mask(from: u8, occ: BitBoardMask) -> BitBoardMask {
    rook_attacks_mask(from, occ) | bishop_attacks_mask(from, occ)
}

#[inline]
pub fn knight_attacks_mask(from: u8) -> BitBoardMask {
    BitBoardMask(KNIGHT_ATTACKS[from as usize])
}

#[inline]
pub fn king_attacks_mask(from: u8) -> BitBoardMask {
    BitBoardMask(KING_ATTACKS[from as usize])
}

#[inline]
pub fn pawn_attacks_mask(from: u8, color: Color) -> BitBoardMask {
    BitBoardMask(PAWN_ATTACKS[color as usize][from as usize])
}
