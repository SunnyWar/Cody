// src/core/bitboardmask.rs
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};
use std::ops::{Shl, Shr};

use crate::core::square::Square;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitBoardMask(pub u64);

impl BitBoardMask {
    #[inline]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn from_square(sq: Square) -> Self {
        Self(1u64 << (sq as u8))
    }

    pub fn set(&mut self, sq: Square) {
        self.0 |= 1u64 << sq.index();
    }

    pub fn clear(&mut self, sq: Square) {
        self.0 &= !(1u64 << sq.index());
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
    pub const fn count_ones(self) -> u32 {
        self.0.count_ones()
    }

    #[inline]
    pub const fn is_singleton(self) -> bool {
        self.count_ones() == 1
    }

    #[inline]
    pub const fn is_nonempty(self) -> bool {
        self.0 != 0
    }

    #[inline]
    pub fn contains_square(self, sq: Square) -> bool {
        (self.0 >> sq.index()) & 1 != 0
    }

    /// Returns an iterator over all set squares in this bitboard.
    #[inline]
    pub fn squares(self) -> impl Iterator<Item = Square> {
        let mut bb = self.0;
        std::iter::from_fn(move || {
            if bb == 0 {
                None
            } else {
                let sq = bb.trailing_zeros() as u8;
                bb &= bb - 1; // clear the lowest set bit
                Square::try_from_index(sq)
            }
        })
    }

    #[inline]
    pub fn first_square(self) -> Option<Square> {
        if self.0 == 0 {
            None
        } else {
            let idx = self.0.trailing_zeros() as u8;
            Square::try_from_index(idx)
        }
    }

    pub fn contains(&self, sq: Square) -> bool {
        self.0 & (1u64 << (sq as u8)) != 0
    }

    pub const fn shift_left(self, n: u32) -> Self {
        BitBoardMask(self.0 << n)
    }

    pub const fn shift_right(self, n: u32) -> Self {
        BitBoardMask(self.0 >> n)
    }

    pub const fn or(self, other: Self) -> Self {
        BitBoardMask(self.0 | other.0)
    }

    pub const fn and(self, other: Self) -> Self {
        BitBoardMask(self.0 & other.0)
    }

    pub const fn xor(self, other: Self) -> Self {
        BitBoardMask(self.0 ^ other.0)
    }

    pub const fn not(self) -> Self {
        BitBoardMask(!self.0)
    }

    pub const fn from_squares(squares: &[Square]) -> Self {
        let mut mask = 0u64;
        let mut i = 0;
        while i < squares.len() {
            mask |= 1u64 << (squares[i] as u8);
            i += 1;
        }
        BitBoardMask(mask)
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

impl From<Square> for BitBoardMask {
    fn from(sq: Square) -> Self {
        BitBoardMask::from_square(sq)
    }
}

impl From<u64> for BitBoardMask {
    fn from(bits: u64) -> Self {
        BitBoardMask(bits)
    }
}

impl BitBoardMask {
    pub const fn subray_left(self, origin: BitBoardMask) -> BitBoardMask {
        let mut ray = BitBoardMask::empty();
        let mut probe = origin.shift_right(1);
        while probe.0 != 0 {
            ray = ray.or(probe);
            if self.and(probe).is_nonempty() {
                break;
            }
            probe = probe.shift_right(1);
        }
        ray
    }

    pub const fn subray_right(self, origin: BitBoardMask) -> BitBoardMask {
        let mut ray = BitBoardMask::empty();
        let mut probe = origin.shift_left(1);

        let mut count = 0;
        while probe.0 != 0 && count < 64 {
            ray = ray.or(probe);
            if self.and(probe).is_nonempty() {
                break;
            }
            probe = probe.shift_left(1);

            count += 1;
        }
        ray
    }

    pub const fn subray_up(self, origin: BitBoardMask) -> BitBoardMask {
        let mut ray = BitBoardMask::empty();
        let mut probe = origin.shift_left(8);
        while probe.0 != 0 {
            ray = ray.or(probe);
            if self.and(probe).is_nonempty() {
                break;
            }
            probe = probe.shift_left(8);
        }
        ray
    }

    pub const fn subray_down(self, origin: BitBoardMask) -> BitBoardMask {
        let mut ray = BitBoardMask::empty();
        let mut probe = origin.shift_right(8);
        while probe.0 != 0 {
            ray = ray.or(probe);
            if self.and(probe).is_nonempty() {
                break;
            }
            probe = probe.shift_right(8);
        }
        ray
    }

    pub const fn subray_up_right(self, origin: BitBoardMask) -> BitBoardMask {
        let mut ray = BitBoardMask::empty();
        let mut probe = origin.shift_left(9);
        while probe.0 != 0 {
            ray = ray.or(probe);
            if self.and(probe).is_nonempty() {
                break;
            }
            probe = probe.shift_left(9);
        }
        ray
    }

    pub const fn subray_down_left(self, origin: BitBoardMask) -> BitBoardMask {
        let mut ray = BitBoardMask::empty();
        let mut probe = origin.shift_right(9);
        while probe.0 != 0 {
            ray = ray.or(probe);
            if self.and(probe).is_nonempty() {
                break;
            }
            probe = probe.shift_right(9);
        }
        ray
    }

    pub const fn subray_up_left(self, origin: BitBoardMask) -> BitBoardMask {
        let mut ray = BitBoardMask::empty();
        let mut probe = origin.shift_left(7);
        while probe.0 != 0 {
            ray = ray.or(probe);
            if self.and(probe).is_nonempty() {
                break;
            }
            probe = probe.shift_left(7);
        }
        ray
    }

    pub const fn subray_down_right(self, origin: BitBoardMask) -> BitBoardMask {
        let mut ray = BitBoardMask::empty();
        let mut probe = origin.shift_right(7);
        while probe.0 != 0 {
            ray = ray.or(probe);
            if self.and(probe).is_nonempty() {
                break;
            }
            probe = probe.shift_right(7);
        }
        ray
    }
}
