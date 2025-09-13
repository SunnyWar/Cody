// src/core/bitboard.rs
#![allow(long_running_const_eval)]

use crate::{
    core::{bitboardmask::BitBoardMask, piece::Color, square::Square},
    generated::{
        BISHOP_ATTACKS, KING_ATTACKS, KNIGHT_ATTACKS, NOT_FILE_A, NOT_FILE_H, ROOK_ATTACKS,
    },
};

pub const BOARD_SIZE: usize = 8;
const NUM_SQUARES: usize = BOARD_SIZE * BOARD_SIZE;

pub struct BitIter(u64);

impl Iterator for BitIter {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            None
        } else {
            let sq = self.0.trailing_zeros() as u8;
            self.0 &= self.0 - 1; // clear LS1B
            Some(sq)
        }
    }
}

#[inline]
pub const fn occupancy_to_index(occupancy: BitBoardMask, mask: BitBoardMask) -> usize {
    let mut index = 0usize;
    let mut bit_index = 0;

    let occupancy_val = occupancy.0;
    let mut mask_val = mask.0;

    while mask_val != 0 {
        let lsb = mask_val & mask_val.wrapping_neg();
        if occupancy_val & lsb != 0 {
            index |= 1 << bit_index;
        }
        mask_val &= mask_val - 1;
        bit_index += 1;
    }

    index
}

#[inline]
pub fn king_attacks(square: Square) -> BitBoardMask {
    KING_ATTACKS[square.index()]
}

#[inline]
pub fn knight_attacks(square: Square) -> BitBoardMask {
    KNIGHT_ATTACKS[square.index()]
}

#[inline]
pub fn rook_attacks(sq: Square, occ_bb: BitBoardMask) -> BitBoardMask {
    let mask_bb = ROOK_MASKS[sq.index()];
    let idx = occupancy_to_index(occ_bb, mask_bb);
    ROOK_ATTACKS[sq.index()][idx]
}

#[inline]
pub fn bishop_attacks(sq: Square, occ_bb: BitBoardMask) -> BitBoardMask {
    let mask_bb = BISHOP_MASKS[sq.index()];
    let idx = occupancy_to_index(occ_bb, mask_bb);
    BISHOP_ATTACKS[sq.index()][idx]
}

#[inline]
const fn rook_mask(square: Square) -> BitBoardMask {
    let rank_mask = square.rank_mask();
    let file_mask = square.file_mask();
    let origin = square.bit();

    rank_mask.or(file_mask).and(origin.not())
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

#[inline]
pub fn rook_attacks_from(square: Square, occupancy: BitBoardMask) -> BitBoardMask {
    let mask = ROOK_MASKS[square.index()];
    let index = occupancy_to_index(occupancy, mask);
    ROOK_ATTACKS[square.index()][index]
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
    pub const fn diagonal_for(square: Square) -> BitBoardMask {
        DIAGONAL_MASKS[square.index()]
    }

    pub const fn antidiagonal_for(square: Square) -> BitBoardMask {
        ANTIDIAGONAL_MASKS[square.index()]
    }
}

#[inline]
const fn bishop_mask(square: Square) -> BitBoardMask {
    BitBoardMask::diagonal_for(square).or(BitBoardMask::antidiagonal_for(square))
}

#[inline]
pub const fn bishop_attacks_from(square: Square, occupancy: BitBoardMask) -> BitBoardMask {
    let origin = square.bit();

    // Diagonal directions
    let diag_mask = DIAGONAL_MASKS[square.index()];
    let diag_blockers = occupancy.and(diag_mask);
    let ne = diag_blockers.subray_up_right(origin);
    let sw = diag_blockers.subray_down_left(origin);
    let diag_attacks = ne.or(sw);

    // Anti-diagonal directions
    let anti_mask = ANTIDIAGONAL_MASKS[square.index()];
    let anti_blockers = occupancy.and(anti_mask);
    let nw = anti_blockers.subray_up_left(origin);
    let se = anti_blockers.subray_down_right(origin);
    let anti_attacks = nw.or(se);

    diag_attacks.or(anti_attacks)
}

#[inline]
const fn pawn_attacks_from(square: Square, color: Color) -> BitBoardMask {
    let bb = square.bit();

    match color {
        Color::White => {
            let nw_attack = bb.shift_left(7).and(NOT_FILE_H);
            let ne_attack = bb.shift_left(9).and(NOT_FILE_A);
            nw_attack.or(ne_attack)
        }
        Color::Black => {
            let sw_attack = bb.shift_right(9).and(NOT_FILE_H);
            let se_attack = bb.shift_right(7).and(NOT_FILE_A);
            sw_attack.or(se_attack)
        }
    }
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
