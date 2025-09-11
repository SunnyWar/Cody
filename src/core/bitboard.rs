// src/core/bitboard.rs
#![allow(long_running_const_eval)]

use crate::core::{bitboardmask::BitBoardMask, piece::Color, square::Square};

// File masks to prevent wrap-around when shifting
const NOT_FILE_A: BitBoardMask = BitBoardMask(0xfefefefefefefefe);
const NOT_FILE_AB: BitBoardMask = BitBoardMask(0xfcfcfcfcfcfcfcfc);
const NOT_FILE_H: BitBoardMask = BitBoardMask(0x7f7f7f7f7f7f7f7f);
const NOT_FILE_GH: BitBoardMask = BitBoardMask(0x3f3f3f3f3f3f3f3f);

pub const BOARD_SIZE: usize = 8;
const NUM_SQUARES: usize = BOARD_SIZE * BOARD_SIZE;
const EMPTY: u64 = 0;

const MAX_ROOK_OCCUPANCY_VARIATIONS: usize = 1 << 12;

const LIGHT_SQUARES: u64 = {
    let mut mask = EMPTY;
    let mut sq: usize = 0;
    while sq < NUM_SQUARES {
        let rank: i8 = (sq / BOARD_SIZE) as i8;
        let file: i8 = (sq % BOARD_SIZE) as i8;
        if (rank + file) % 2 == 0 {
            mask |= 1u64 << sq;
        }
        sq += 1;
    }
    mask
};

const DARK_SQUARES: u64 = !LIGHT_SQUARES;

// TODO: Change to use enum instead of index
pub const SQUARE_COLOR_MASK: [u64; NUM_SQUARES] = {
    let mut arr = [0u64; NUM_SQUARES];
    let mut sq = 0;
    while sq < NUM_SQUARES {
        arr[sq] = if (sq / BOARD_SIZE + sq % BOARD_SIZE).is_multiple_of(2) {
            LIGHT_SQUARES
        } else {
            DARK_SQUARES
        };
        sq += 1;
    }
    arr
};

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

    // Unwrap the inner u64s for the loop, but only inside this function
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

const fn square_index(rank: i8, file: i8) -> usize {
    debug_assert!(rank >= 0 && rank < BOARD_SIZE as i8);
    debug_assert!(file >= 0 && file < BOARD_SIZE as i8);
    rank as usize * BOARD_SIZE + file as usize
}

#[inline]
pub const fn gen_king_attacks(square: Square) -> BitBoardMask {
    let mut attack_mask: u64 = 0;
    let rank = square.rank();
    let file = square.file();

    const KING_OFFSETS: [(i8, i8); 8] = [
        (1, 0),
        (1, 1),
        (0, 1),
        (-1, 1),
        (-1, 0),
        (-1, -1),
        (0, -1),
        (1, -1),
    ];

    let mut offset_index = 0;
    while offset_index < KING_OFFSETS.len() {
        let (rank_offset, file_offset) = KING_OFFSETS[offset_index];
        let target_rank = rank as i8 + rank_offset;
        let target_file = file as i8 + file_offset;

        if target_rank >= 0
            && target_rank < BOARD_SIZE as i8
            && target_file >= 0
            && target_file < BOARD_SIZE as i8
        {
            let target_index = square_index(target_rank, target_file);
            attack_mask |= 1u64 << target_index;
        }

        offset_index += 1;
    }

    BitBoardMask(attack_mask)
}

pub const KING_ATTACKS: [BitBoardMask; NUM_SQUARES] = {
    let mut table = [BitBoardMask(0); NUM_SQUARES];
    let squares = Square::all_array();
    let mut i = 0;
    while i < NUM_SQUARES {
        let sq = squares[i];
        table[i] = gen_king_attacks(sq);
        i += 1;
    }
    table
};

#[inline]
pub const fn knight_attacks_for(square: Square) -> BitBoardMask {
    let origin = square.bit();

    // Horizontal shifts with file exclusions
    let h1 = origin
        .shift_right(1)
        .and(NOT_FILE_H)
        .or(origin.shift_left(1).and(NOT_FILE_A));

    let h2 = origin
        .shift_right(2)
        .and(NOT_FILE_GH)
        .or(origin.shift_left(2).and(NOT_FILE_AB));

    // Vertical shifts to complete the L-shape
    let v1 = h1.shift_left(16).or(h1.shift_right(16));
    let v2 = h2.shift_left(8).or(h2.shift_right(8));

    v1.or(v2)
}

// TODO - put all attacks in attack tables struct
/* pub struct AttackTables;

impl AttackTables {
    pub fn knight(sq: Square) -> BitBoardMask {
        KNIGHT_ATTACKS[sq as usize]
    }
}
 */

pub const KNIGHT_ATTACKS: [BitBoardMask; NUM_SQUARES] = {
    let mut table = [BitBoardMask(0); NUM_SQUARES];
    let squares = Square::all_array();
    let mut i = 0;
    while i < NUM_SQUARES {
        table[i] = knight_attacks_for(squares[i]);
        i += 1;
    }
    table
};

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
pub const fn rook_attacks_from(square: Square, occupancy: BitBoardMask) -> BitBoardMask {
    let origin = square.bit();

    // Horizontal (rank) attacks
    let rank_mask = square.rank_mask();
    let blockers = occupancy.and(rank_mask);
    let left = blockers.subray_left(origin);
    let right = blockers.subray_right(origin);
    let rank_attacks = BitBoardMask(left.0 | right.0);

    // Vertical (file) attacks
    let file_mask = square.file_mask();
    let blockers = occupancy.and(file_mask);
    let up = blockers.subray_up(origin);
    let down = blockers.subray_down(origin);
    let file_attacks = up.or(down);

    rank_attacks.or(file_attacks)
}

pub static ROOK_ATTACKS: [[BitBoardMask; MAX_ROOK_OCCUPANCY_VARIATIONS]; NUM_SQUARES] = {
    let mut table = [[BitBoardMask::empty(); MAX_ROOK_OCCUPANCY_VARIATIONS]; NUM_SQUARES];
    let squares = Square::all_array();
    let mut sq_idx = 0;

    while sq_idx < NUM_SQUARES {
        let square = squares[sq_idx];
        let mask = ROOK_MASKS[sq_idx];
        let mask_val = mask.0;
        let occupancy_variations = 1usize << mask_val.count_ones();
        let max_variations = if occupancy_variations > MAX_ROOK_OCCUPANCY_VARIATIONS {
            MAX_ROOK_OCCUPANCY_VARIATIONS
        } else {
            occupancy_variations
        };

        let mut index = 0;
        while index < max_variations {
            let mut occupancy_val = 0u64;
            let mut bits = mask_val;
            let subset = index;
            let mut bit_index = 0;

            while bits != 0 {
                let lsb = bits & bits.wrapping_neg();
                if (subset >> bit_index) & 1 != 0 {
                    occupancy_val |= lsb;
                }
                bits &= bits - 1;
                bit_index += 1;
            }

            let occupancy = BitBoardMask(occupancy_val);
            table[sq_idx][index] = rook_attacks_from(square, occupancy);

            index += 1;
        }

        sq_idx += 1;
    }

    table
};

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
        DIAGONAL_MASKS[square.idx()]
    }

    pub const fn antidiagonal_for(square: Square) -> BitBoardMask {
        ANTIDIAGONAL_MASKS[square.idx()]
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
    let diag_mask = DIAGONAL_MASKS[square.idx()];
    let diag_blockers = occupancy.and(diag_mask);
    let ne = diag_blockers.subray_up_right(origin);
    let sw = diag_blockers.subray_down_left(origin);
    let diag_attacks = ne.or(sw);

    // Anti-diagonal directions
    let anti_mask = ANTIDIAGONAL_MASKS[square.idx()];
    let anti_blockers = occupancy.and(anti_mask);
    let nw = anti_blockers.subray_up_left(origin);
    let se = anti_blockers.subray_down_right(origin);
    let anti_attacks = nw.or(se);

    diag_attacks.or(anti_attacks)
}

pub static BISHOP_ATTACKS: [[BitBoardMask; 512]; NUM_SQUARES] = {
    let mut table = [[BitBoardMask::empty(); 512]; NUM_SQUARES];
    let squares = Square::all_array();
    let mut sq_idx = 0;

    while sq_idx < NUM_SQUARES {
        let square = squares[sq_idx];
        let mask = BISHOP_MASKS[sq_idx];
        let mask_val = mask.0;
        let occupancy_variations = 1usize << mask_val.count_ones();
        let max_variations = if occupancy_variations > 512 {
            512
        } else {
            occupancy_variations
        };

        let mut index = 0;
        while index < max_variations {
            let mut occupancy_val = 0u64;
            let mut bits = mask_val;
            let subset = index;
            let mut bit_index = 0;

            while bits != 0 {
                let lsb = bits & bits.wrapping_neg();
                if (subset >> bit_index) & 1 != 0 {
                    occupancy_val |= lsb;
                }
                bits &= bits - 1;
                bit_index += 1;
            }

            let occupancy = BitBoardMask(occupancy_val);
            table[sq_idx][index] = bishop_attacks_from(square, occupancy);

            index += 1;
        }

        sq_idx += 1;
    }

    table
};

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
