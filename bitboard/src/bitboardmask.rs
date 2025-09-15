use std::arch::x86_64::*;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};
use std::ops::{Shl, Shr};

use crate::Square;

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

    #[inline]
    pub fn set(&mut self, sq: Square) {
        self.0 |= 1u64 << sq.index();
    }

    #[inline]
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
        // Faster than count_ones() == 1
        self.0 != 0 && (self.0 & (self.0 - 1)) == 0
    }

    #[inline]
    pub const fn is_nonempty(self) -> bool {
        self.0 != 0
    }

    #[inline]
    pub fn contains_square(self, sq: Square) -> bool {
        (self.0 & (1u64 << sq.index())) != 0
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

    #[inline]
    pub fn contains(&self, sq: Square) -> bool {
        self.0 & (1u64 << (sq as u8)) != 0
    }

    #[inline]
    pub const fn shift_left(self, n: u32) -> Self {
        BitBoardMask(self.0 << n)
    }

    #[inline]
    pub const fn shift_right(self, n: u32) -> Self {
        BitBoardMask(self.0 >> n)
    }

    #[inline]
    pub const fn or(self, other: Self) -> Self {
        BitBoardMask(self.0 | other.0)
    }

    #[inline]
    pub const fn and(self, other: Self) -> Self {
        BitBoardMask(self.0 & other.0)
    }

    #[inline]
    pub const fn xor(self, other: Self) -> Self {
        BitBoardMask(self.0 ^ other.0)
    }

    #[inline]
    pub const fn not(self) -> Self {
        BitBoardMask(!self.0)
    }

    #[inline]
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

// Standard trait implementations remain the same...
impl Shl<i8> for BitBoardMask {
    type Output = Self;
    #[inline]
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
    #[inline]
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
    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        BitBoardMask(self.0 & rhs.0)
    }
}

impl BitAndAssign for BitBoardMask {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOr for BitBoardMask {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        BitBoardMask(self.0 | rhs.0)
    }
}

impl BitOrAssign for BitBoardMask {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl Not for BitBoardMask {
    type Output = Self;
    #[inline]
    fn not(self) -> Self::Output {
        BitBoardMask(!self.0)
    }
}

impl From<Square> for BitBoardMask {
    #[inline]
    fn from(sq: Square) -> Self {
        BitBoardMask::from_square(sq)
    }
}

impl From<u64> for BitBoardMask {
    #[inline]
    fn from(bits: u64) -> Self {
        BitBoardMask(bits)
    }
}

// Pre-computed ray masks for ultra-fast lookups (if needed)
// These would be at module level, not in impl block
// static RAY_MASKS: [[u64; 64]; 8] = [[0; 64]; 8]; // [direction][square]

// Optimized ray casting functions
impl BitBoardMask {
    #[inline]
    pub fn subray_left_optimized(self, origin_sq: u8) -> BitBoardMask {
        // Use bit manipulation for horizontal rays
        let rank_mask = 0xFFu64 << (origin_sq & 56); // Get rank mask
        let occupied_rank = self.0 & rank_mask;
        let origin_bit = 1u64 << origin_sq;

        // Find blocking piece to the left
        let left_pieces = occupied_rank & (origin_bit - 1);
        if left_pieces == 0 {
            // No blockers, ray extends to left edge of rank
            BitBoardMask((origin_bit - 1) & rank_mask)
        } else {
            // Find rightmost blocker (MSB of left_pieces)
            let blocker_pos = 63 - left_pieces.leading_zeros();
            let ray_mask = (origin_bit - 1) & !((1u64 << (blocker_pos + 1)) - 1);
            BitBoardMask(ray_mask)
        }
    }

    // Traditional optimized version for const contexts
    pub const fn subray_left(self, origin: BitBoardMask) -> BitBoardMask {
        // Get the origin square position
        let origin_pos = origin.0.trailing_zeros();
        if origin_pos >= 64 {
            return BitBoardMask::empty();
        }

        // Calculate rank mask to prevent wrapping
        let rank = origin_pos / 8;
        let rank_mask = 0xFFu64 << (rank * 8);

        // Use bit manipulation to find blocking piece
        let occupied_in_rank = self.0 & rank_mask;
        let origin_bit = 1u64 << origin_pos;

        // Find first blocker to the left using bit tricks
        let left_mask = origin_bit - 1; // All bits to the right of origin
        let left_occupied = occupied_in_rank & left_mask & rank_mask;

        if left_occupied == 0 {
            // No blockers, ray goes to edge of rank
            return BitBoardMask(left_mask & rank_mask);
        }

        // Find rightmost blocker (closest to origin)
        let blocker_pos = 63 - left_occupied.leading_zeros();
        let ray_mask = (origin_bit - 1) & !((1u64 << (blocker_pos + 1)) - 1);

        BitBoardMask(ray_mask & rank_mask)
    }

    pub const fn subray_right(self, origin: BitBoardMask) -> BitBoardMask {
        let origin_pos = origin.0.trailing_zeros();
        if origin_pos >= 64 {
            return BitBoardMask::empty();
        }

        let rank = origin_pos / 8;
        let rank_mask = 0xFFu64 << (rank * 8);
        let occupied_in_rank = self.0 & rank_mask;
        let origin_bit = 1u64 << origin_pos;

        // Find first blocker to the right
        let right_mask = !((origin_bit << 1) - 1); // All bits to the left of origin
        let right_occupied = occupied_in_rank & right_mask & rank_mask;

        if right_occupied == 0 {
            // No blockers, ray goes to edge of rank
            let edge_mask = !(origin_bit | (origin_bit - 1));
            return BitBoardMask(edge_mask & rank_mask);
        }

        // Find leftmost blocker (closest to origin)
        let blocker_pos = right_occupied.trailing_zeros();
        let ray_mask = ((1u64 << blocker_pos) - 1) & !((origin_bit << 1) - 1);

        BitBoardMask(ray_mask & rank_mask)
    }

    // Optimized vertical rays using similar techniques
    pub const fn subray_up(self, origin: BitBoardMask) -> BitBoardMask {
        let origin_pos = origin.0.trailing_zeros();
        if origin_pos >= 64 {
            return BitBoardMask::empty();
        }

        let file = origin_pos % 8;
        let file_mask = 0x0101010101010101u64 << file;
        let occupied_in_file = self.0 & file_mask;
        let origin_bit = 1u64 << origin_pos;

        // Find first blocker upward
        let up_mask = !((origin_bit << 8) - 1); // All squares above
        let up_occupied = occupied_in_file & up_mask;

        if up_occupied == 0 {
            let edge_mask = !(origin_bit | ((origin_bit << 8) - 1));
            return BitBoardMask(edge_mask & file_mask);
        }

        let blocker_pos = up_occupied.trailing_zeros();
        let ray_mask = ((1u64 << blocker_pos) - 1) & !((origin_bit << 8) - 1);

        BitBoardMask(ray_mask & file_mask)
    }

    pub const fn subray_down(self, origin: BitBoardMask) -> BitBoardMask {
        let origin_pos = origin.0.trailing_zeros();
        if origin_pos >= 64 {
            return BitBoardMask::empty();
        }

        let file = origin_pos % 8;
        let file_mask = 0x0101010101010101u64 << file;
        let occupied_in_file = self.0 & file_mask;
        let origin_bit = 1u64 << origin_pos;

        // Find first blocker downward
        let down_mask = origin_bit - 1; // All squares below
        let down_occupied = occupied_in_file & down_mask;

        if down_occupied == 0 {
            return BitBoardMask(down_mask & file_mask);
        }

        let blocker_pos = 63 - down_occupied.leading_zeros();
        let ray_mask = (origin_bit - 1) & !((1u64 << (blocker_pos + 1)) - 1);

        BitBoardMask(ray_mask & file_mask)
    }

    // For diagonal rays, we can use similar bit manipulation techniques
    // but they're more complex due to diagonal wrapping issues
    pub const fn subray_up_right(self, origin: BitBoardMask) -> BitBoardMask {
        // Simplified implementation - full optimization would require diagonal masks
        let mut ray = BitBoardMask::empty();
        let mut probe = origin.shift_left(9);

        // Unroll first few iterations for better performance
        if probe.0 != 0 && (probe.0 & 0xFEFEFEFEFEFEFEFE) != 0 {
            // Not on left edge
            ray = ray.or(probe);
            if self.and(probe).is_nonempty() {
                return ray;
            }
            probe = probe.shift_left(9);

            if probe.0 != 0 && (probe.0 & 0xFEFEFEFEFEFEFEFE) != 0 {
                ray = ray.or(probe);
                if self.and(probe).is_nonempty() {
                    return ray;
                }
                probe = probe.shift_left(9);
            }
        }

        // Continue with loop for remaining squares
        while probe.0 != 0 && (probe.0 & 0xFEFEFEFEFEFEFEFE) != 0 {
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

        // Unroll and add edge detection
        while probe.0 != 0 && (probe.0 & 0x7F7F7F7F7F7F7F7F) != 0 {
            // Not on right edge
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

        while probe.0 != 0 && (probe.0 & 0x7F7F7F7F7F7F7F7F) != 0 {
            // Not on right edge
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

        while probe.0 != 0 && (probe.0 & 0xFEFEFEFEFEFEFEFE) != 0 {
            // Not on left edge
            ray = ray.or(probe);
            if self.and(probe).is_nonempty() {
                break;
            }
            probe = probe.shift_right(7);
        }
        ray
    }

    // BMI2 optimized version for runtime use
    #[cfg(target_arch = "x86_64")]
    #[inline]
    pub fn subray_horizontal_bmi2(self, origin_sq: u8, direction: bool) -> BitBoardMask {
        unsafe {
            if std::arch::is_x86_feature_detected!("bmi2") {
                let rank = (origin_sq / 8) as u64;
                let rank_mask = 0xFFu64 << (rank * 8);
                let occupied = _pext_u64(self.0, rank_mask) as u8;
                let file = origin_sq % 8;

                // Use lookup tables or bit manipulation to compute ray
                // This is where you'd implement hyperbola quintessence with BMI2
                let ray = if direction {
                    // Right direction
                    occupied & ((1u8 << file) - 1)
                } else {
                    // Left direction
                    occupied & !((1u8 << (file + 1)) - 1)
                };

                BitBoardMask(_pdep_u64(ray as u64, rank_mask))
            } else {
                // Fallback to const version
                if direction {
                    self.subray_right(BitBoardMask(1u64 << origin_sq))
                } else {
                    self.subray_left(BitBoardMask(1u64 << origin_sq))
                }
            }
        }
    }
}
