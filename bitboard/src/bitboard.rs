// bitboard/src/bitboard.rs
#![allow(long_running_const_eval)]

use crate::BitBoardMask;
use crate::Square;
use crate::constants::BOARD_SIZE;
use crate::constants::NUM_SQUARES;
use crate::piece::Color;
use crate::tables::bishop_attack::BISHOP_ATTACKS;
use crate::tables::file_masks::NOT_FILE_A;
use crate::tables::file_masks::NOT_FILE_H;
use crate::tables::king_attack::KING_ATTACKS;
use crate::tables::knight_attack::KNIGHT_ATTACKS;
use crate::tables::rook_attack::ROOK_ATTACKS;

pub struct BitIter(u64);

impl Iterator for BitIter {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            None
        } else {
            #[allow(clippy::cast_possible_truncation)]
            let sq = crate::intrinsics::trailing_zeros_nonzero(self.0) as u8;
            self.0 = crate::intrinsics::blsr_nonzero(self.0); // clear LS1B
            Some(sq)
        }
    }
}

#[must_use]
pub fn occupancy_to_index(occupancy: BitBoardMask, mask: BitBoardMask) -> usize {
    occupancy_to_index_u64(occupancy.0, mask.0)
}

#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn occupancy_to_index_u64(occupancy: u64, mask: u64) -> usize {
    // Use PEXT (Parallel Bits Extract) via our intrinsics module
    // This will use hardware BMI2 when available, software fallback otherwise
    crate::intrinsics::pext(occupancy, mask) as usize
}

#[must_use]
pub const fn king_attacks(square: Square) -> BitBoardMask {
    KING_ATTACKS[square.index()]
}

#[must_use]
pub const fn knight_attacks(square: Square) -> BitBoardMask {
    KNIGHT_ATTACKS[square.index()]
}

#[must_use]
pub fn rook_attacks(sq: Square, occ_bb: BitBoardMask) -> BitBoardMask {
    let mask_bb = ROOK_MASKS[sq.index()];
    let idx = occupancy_to_index_u64(occ_bb.0, mask_bb.0);
    ROOK_ATTACKS[sq.index()][idx]
}

#[must_use]
pub fn bishop_attacks(sq: Square, occ_bb: BitBoardMask) -> BitBoardMask {
    let mask_bb = BISHOP_MASKS[sq.index()];
    let idx = occupancy_to_index_u64(occ_bb.0, mask_bb.0);
    BISHOP_ATTACKS[sq.index()][idx]
}

const fn rook_mask(square: Square) -> BitBoardMask {
    let rank_mask = square.rank_mask();
    let file_mask = square.file_mask();
    let origin = square.bitboard();
    BitBoardMask((rank_mask.0 | file_mask.0) & !origin.0)
}

pub const ROOK_MASKS: [BitBoardMask; NUM_SQUARES] = {
    let mut table = [BitBoardMask::empty(); NUM_SQUARES];
    let squares = Square::all_array();
    let mut i = 0;
    while i < NUM_SQUARES {
        let sq = squares[i];
        table[i] = rook_mask(sq);
        i += 1;
    }
    table
};

#[must_use]
pub fn rook_attacks_from(square: Square, occupancy: BitBoardMask) -> BitBoardMask {
    let sq = square.index();

    // SAFETY: `sq` is derived from `Square` and therefore in 0..64.
    let mask = unsafe { *ROOK_MASKS.get_unchecked(sq) };
    let index = occupancy_to_index_u64(occupancy.0, mask.0);

    // SAFETY: `sq` is in-bounds and `index` is produced from rook occupancy mask
    // via `occupancy_to_index`, matching the precomputed attack table dimensions.
    unsafe { *ROOK_ATTACKS.get_unchecked(sq).get_unchecked(index) }
}

pub const BISHOP_MASKS: [BitBoardMask; NUM_SQUARES] = {
    let mut masks = [BitBoardMask::empty(); NUM_SQUARES];
    let squares = Square::all_array();
    let mut i = 0;

    while i < NUM_SQUARES {
        let sq = squares[i];
        masks[i] = bishop_mask(sq);
        i += 1;
    }

    masks
};

impl BitBoardMask {
    #[must_use]
    pub const fn diagonal_for(square: Square) -> BitBoardMask {
        DIAGONAL_MASKS[square.index()]
    }

    #[must_use]
    pub const fn antidiagonal_for(square: Square) -> BitBoardMask {
        ANTIDIAGONAL_MASKS[square.index()]
    }
}

const fn bishop_mask(square: Square) -> BitBoardMask {
    let d = BitBoardMask::diagonal_for(square);
    let a = BitBoardMask::antidiagonal_for(square);
    BitBoardMask(d.0 | a.0)
}

/// Fast bishop attacks using magic bitboard lookup tables.
/// This is the hot-path version used in move generation.
pub fn bishop_attacks_from(square: Square, occupancy: BitBoardMask) -> BitBoardMask {
    let sq = square.index();

    // SAFETY: `sq` is derived from `Square` and therefore in 0..64.
    let mask = unsafe { *BISHOP_MASKS.get_unchecked(sq) };
    let index = occupancy_to_index_u64(occupancy.0, mask.0);

    // SAFETY: `sq` is in-bounds and `index` is produced from bishop occupancy
    // mask via `occupancy_to_index`, matching the precomputed attack table.
    unsafe { *BISHOP_ATTACKS.get_unchecked(sq).get_unchecked(index) }
}

/// Const version for compile-time bishop attack computation.
/// Kept for initializing const tables but not used in hot path.
#[must_use]
pub const fn bishop_attacks_from_const(square: Square, occupancy: BitBoardMask) -> BitBoardMask {
    let origin = square.bitboard();
    // Diagonal directions
    let diag_mask = DIAGONAL_MASKS[square.index()];
    let diag_blockers = BitBoardMask(occupancy.0 & diag_mask.0);
    let ne = diag_blockers.subray_up_right(origin);
    let sw = diag_blockers.subray_down_left(origin);
    let diag_attacks = BitBoardMask(ne.0 | sw.0);
    // Anti-diagonal directions
    let anti_mask = ANTIDIAGONAL_MASKS[square.index()];
    let anti_blockers = BitBoardMask(occupancy.0 & anti_mask.0);
    let nw = anti_blockers.subray_up_left(origin);
    let se = anti_blockers.subray_down_right(origin);
    let anti_attacks = BitBoardMask(nw.0 | se.0);
    BitBoardMask(diag_attacks.0 | anti_attacks.0)
}

const fn pawn_attacks_from(square: Square, color: Color) -> BitBoardMask {
    let bb = square.bitboard();
    match color {
        Color::White => {
            let nw_target = BitBoardMask((bb.0 << 7) & NOT_FILE_H.0);
            let ne_target = BitBoardMask((bb.0 << 9) & NOT_FILE_A.0);
            BitBoardMask(nw_target.0 | ne_target.0)
        }
        Color::Black => {
            let sw_target = BitBoardMask((bb.0 >> 9) & NOT_FILE_H.0);
            let se_target = BitBoardMask((bb.0 >> 7) & NOT_FILE_A.0);
            BitBoardMask(sw_target.0 | se_target.0)
        }
    }
}

#[must_use]
pub const fn pawn_attacks_to(sq: Square, attacker_color: Color) -> BitBoardMask {
    // Reverse the direction: squares that can attack `sq` for this color.
    // Direct index computation avoids opposite().index() call overhead.
    // Since Color::White=0 and Color::Black=1, we compute XOR with 1 to flip.
    let defender_idx = (attacker_color as usize) ^ 1;
    PAWN_ATTACKS[defender_idx][sq.index()]
}

const fn generate_attacks_for_color(color: Color) -> [BitBoardMask; NUM_SQUARES] {
    let mut attacks = [BitBoardMask::empty(); NUM_SQUARES];
    let squares = Square::all_array();

    let mut i = 0;
    while i < NUM_SQUARES {
        attacks[i] = pawn_attacks_from(squares[i], color);
        i += 1;
    }
    attacks
}

pub const PAWN_ATTACKS: [[BitBoardMask; NUM_SQUARES]; 2] = [
    generate_attacks_for_color(Color::White),
    generate_attacks_for_color(Color::Black),
];

pub const DIAGONAL_MASKS: [BitBoardMask; NUM_SQUARES] = {
    let mut table = [BitBoardMask::empty(); NUM_SQUARES];
    let mut sq = 0;
    while sq < NUM_SQUARES {
        let rank = sq / BOARD_SIZE;
        let file = sq % BOARD_SIZE;
        let mut mask: u64 = 0;

        // Walk NE (exclude origin)
        let mut r = rank + 1;
        let mut f = file + 1;
        while r < BOARD_SIZE && f < BOARD_SIZE {
            mask |= 1u64 << (r * BOARD_SIZE + f);
            r += 1;
            f += 1;
        }

        // Walk SW (exclude origin)
        let mut r = rank.wrapping_sub(1);
        let mut f = file.wrapping_sub(1);
        while r < BOARD_SIZE && f < BOARD_SIZE {
            mask |= 1u64 << (r * BOARD_SIZE + f);
            if r == 0 || f == 0 {
                break;
            }
            r -= 1;
            f -= 1;
        }

        table[sq] = BitBoardMask(mask);
        sq += 1;
    }
    table
};

pub const ANTIDIAGONAL_MASKS: [BitBoardMask; NUM_SQUARES] = {
    let mut table = [BitBoardMask::empty(); NUM_SQUARES];
    let mut sq = 0;
    while sq < NUM_SQUARES {
        let rank = sq / BOARD_SIZE;
        let file = sq % BOARD_SIZE;
        let mut mask: u64 = 0;

        // Walk NW (exclude origin)
        let mut r = rank.wrapping_add(1);
        let mut f = file.wrapping_sub(1);
        while r < BOARD_SIZE && f < BOARD_SIZE {
            mask |= 1u64 << (r * BOARD_SIZE + f);
            r += 1;
            if f == 0 {
                break;
            }
            f -= 1;
        }

        // Walk SE (exclude origin)
        let mut r = rank.wrapping_sub(1);
        let mut f = file.wrapping_add(1);
        while r < BOARD_SIZE && f < BOARD_SIZE {
            mask |= 1u64 << (r * BOARD_SIZE + f);
            if r == 0 {
                break;
            }
            r -= 1;
            f += 1;
        }

        table[sq] = BitBoardMask(mask);
        sq += 1;
    }
    table
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_occupancy_to_index_basic() {
        let mask = BitBoardMask(0b10110); // bits at positions 1, 2, and 4
        let occupancy = BitBoardMask(0b10010); // bits at positions 1 and 4 are set

        // The mask bits are: [bit 1, bit 2, bit 4]
        // Occupancy has bits 1 and 4 set, so index should be: 1 (bit 0) + 0 (bit 1) + 1
        // (bit 2) = 0b101 = 5
        let index = occupancy_to_index(occupancy, mask);
        assert_eq!(index, 0b101);
    }

    #[test]
    fn test_occupancy_to_index_empty_mask() {
        let mask = BitBoardMask(0);
        let occupancy = BitBoardMask(0b111111);
        let index = occupancy_to_index(occupancy, mask);
        assert_eq!(index, 0);
    }

    #[test]
    fn test_occupancy_to_index_full_match() {
        let mask = BitBoardMask(0b111);
        let occupancy = BitBoardMask(0b111);
        let index = occupancy_to_index(occupancy, mask);
        assert_eq!(index, 0b111);
    }

    #[test]
    fn test_occupancy_to_index_partial_match() {
        let mask = BitBoardMask(0b101010);
        let occupancy = BitBoardMask(0b100000);
        let index = occupancy_to_index(occupancy, mask);
        // Only the highest bit in mask is set in occupancy, which is the 3rd bit in
        // mask order
        assert_eq!(index, 0b100);
    }
}
