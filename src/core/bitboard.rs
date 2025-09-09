// src/core/bitboard.rs

use crate::core::piece::Color;

pub const FILE_A: u64 = 0x0101010101010101;
pub const FILE_B: u64 = 0x0202020202020202;
pub const FILE_C: u64 = 0x0404040404040404;
pub const FILE_D: u64 = 0x0808080808080808;
pub const FILE_E: u64 = 0x1010101010101010;
pub const FILE_F: u64 = 0x2020202020202020;
pub const FILE_G: u64 = 0x4040404040404040;
pub const FILE_H: u64 = 0x8080808080808080;

pub const RANK_1_MASK: u64 = 0x00000000000000FF;
pub const RANK_2_MASK: u64 = 0x000000000000FF00;
pub const RANK_3_MASK: u64 = 0x0000000000FF0000;
pub const RANK_4_MASK: u64 = 0x00000000FF000000;
pub const RANK_5_MASK: u64 = 0x000000FF00000000;
pub const RANK_6_MASK: u64 = 0x0000FF0000000000;
pub const RANK_7_MASK: u64 = 0x00FF000000000000;
pub const RANK_8_MASK: u64 = 0xFF00000000000000;

pub const NORTH: i8 = 8;
pub const SOUTH: i8 = -8;
pub const NORTH_EAST: i8 = 9;
pub const NORTH_WEST: i8 = 7;
pub const SOUTH_EAST: i8 = -7;
pub const SOUTH_WEST: i8 = -9;
pub const DOUBLE_NORTH: i8 = 16;
pub const DOUBLE_SOUTH: i8 = -16;

// File masks to prevent wrap-around when shifting
pub const NOT_FILE_A: u64 = 0xfefefefefefefefe;
pub const NOT_FILE_AB: u64 = 0xfcfcfcfcfcfcfcfc;
pub const NOT_FILE_H: u64 = 0x7f7f7f7f7f7f7f7f;
pub const NOT_FILE_GH: u64 = 0x3f3f3f3f3f3f3f3f;

pub const BOARD_SIZE: usize = 8;
pub const NUM_SQUARES: usize = BOARD_SIZE * BOARD_SIZE;
pub const EMPTY: u64 = 0;

pub const RANK_0: i8 = 0;
pub const FILE_0: i8 = 0;
pub const RANK_MAX: i8 = BOARD_SIZE as i8 - 1;
pub const FILE_MAX: i8 = BOARD_SIZE as i8 - 1;

pub const KING_MOVE_RANGE: i8 = 1;
pub const MAX_ROOK_OCC_VARIATIONS: usize = 1 << 12;

// For rook masks, we stop one square before the edge (exclude outer rank/file)
pub const INNER_MIN: i8 = 1;
pub const INNER_MAX: i8 = BOARD_SIZE as i8 - 2;

// For rook attacks, we can go all the way to the edge
pub const EDGE_MIN: i8 = 0;
pub const EDGE_MAX: i8 = BOARD_SIZE as i8 - 1;

pub const LIGHT_SQUARES: u64 = {
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

pub const DARK_SQUARES: u64 = !LIGHT_SQUARES;

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

#[inline]
pub const fn bit(sq: u8) -> u64 {
    1u64 << sq
}

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

#[inline(always)]
pub fn bit_iter(bb: u64) -> BitIter {
    BitIter(bb)
}

pub const fn occ_to_index(occ: u64, mut mask: u64) -> usize {
    let mut index = 0usize;
    let mut bit_index = 0;
    while mask != 0 {
        let lsb = mask & mask.wrapping_neg();
        if occ & lsb != 0 {
            index |= 1 << bit_index;
        }
        mask &= mask - 1;
        bit_index += 1;
    }
    index
}

pub const fn gen_king_attacks(square: usize) -> u64 {
    let mut attacks = EMPTY;

    // Work in i8 for rank/file math
    let rank: i8 = (square / BOARD_SIZE) as i8;
    let file: i8 = (square % BOARD_SIZE) as i8;

    let mut r = rank - KING_MOVE_RANGE;
    while r <= rank + KING_MOVE_RANGE {
        let mut f = file - KING_MOVE_RANGE;
        while f <= file + KING_MOVE_RANGE {
            if r >= RANK_0 && r <= RANK_MAX && f >= FILE_0 && f <= FILE_MAX {
                let sq: usize = (r as usize) * BOARD_SIZE + (f as usize);
                if sq != square {
                    attacks |= 1u64 << sq;
                }
            }
            f += 1;
        }
        r += 1;
    }

    attacks
}

const fn init_king_attacks() -> [u64; NUM_SQUARES] {
    let mut table = [EMPTY; NUM_SQUARES];
    let mut sq = 0;
    while sq < NUM_SQUARES {
        table[sq] = gen_king_attacks(sq);
        sq += 1;
    }
    table
}
pub const KING_ATTACKS: [u64; 64] = init_king_attacks();

pub const fn knight_attacks_for(sq: u8) -> u64 {
    let b = bit(sq);

    // One file left/right
    let left1 = (b >> 1) & NOT_FILE_H;
    let right1 = (b << 1) & NOT_FILE_A;

    // Two files left/right
    let left2 = (b >> 2) & NOT_FILE_GH;
    let right2 = (b << 2) & NOT_FILE_AB;

    // Horizontal moves
    let h1 = left1 | right1;
    let h2 = left2 | right2;

    // Vertical shifts to complete the L-shape
    (h1 << 16) | (h1 >> 16) | (h2 << 8) | (h2 >> 8)
}

pub const KNIGHT_ATTACKS: [u64; NUM_SQUARES] = {
    let mut table = [EMPTY; NUM_SQUARES];
    let mut sq = 0;
    while sq < NUM_SQUARES {
        table[sq] = knight_attacks_for(sq as u8);
        sq += 1;
    }
    table
};

const fn rook_mask(sq: u8) -> u64 {
    let mut mask = EMPTY;
    let rank = (sq / BOARD_SIZE as u8) as i8;
    let file = (sq % BOARD_SIZE as u8) as i8;

    // North
    let mut r = rank + 1;
    while r <= INNER_MAX {
        mask |= 1u64 << (r * BOARD_SIZE as i8 + file);
        r += 1;
    }
    // South
    r = rank - 1;
    while r >= INNER_MIN {
        mask |= 1u64 << (r * BOARD_SIZE as i8 + file);
        r -= 1;
    }
    // East
    let mut f = file + 1;
    while f <= INNER_MAX {
        mask |= 1u64 << (rank * BOARD_SIZE as i8 + f);
        f += 1;
    }
    // West
    f = file - 1;
    while f >= INNER_MIN {
        mask |= 1u64 << (rank * BOARD_SIZE as i8 + f);
        f -= 1;
    }

    mask
}

pub const ROOK_MASKS: [u64; NUM_SQUARES] = {
    let mut table = [EMPTY; NUM_SQUARES];
    let mut sq = 0;
    while sq < NUM_SQUARES {
        table[sq] = rook_mask(sq as u8);
        sq += 1;
    }
    table
};

pub const fn rook_attacks_from(sq: u8, occ: u64) -> u64 {
    let mut attacks = EMPTY;

    // Work in i8 for rank/file math
    let rank: i8 = (sq as usize / BOARD_SIZE) as i8;
    let file: i8 = (sq as usize % BOARD_SIZE) as i8;

    // North
    let mut r = rank + 1;
    while r <= EDGE_MAX {
        let sq_idx: usize = (r as usize) * BOARD_SIZE + (file as usize);
        attacks |= 1u64 << sq_idx;
        if occ & (1u64 << sq_idx) != 0 {
            break;
        }
        r += 1;
    }

    // South
    r = rank - 1;
    while r >= EDGE_MIN {
        let sq_idx: usize = (r as usize) * BOARD_SIZE + (file as usize);
        attacks |= 1u64 << sq_idx;
        if occ & (1u64 << sq_idx) != 0 {
            break;
        }
        r -= 1;
    }

    // East
    let mut f = file + 1;
    while f <= EDGE_MAX {
        let sq_idx: usize = (rank as usize) * BOARD_SIZE + (f as usize);
        attacks |= 1u64 << sq_idx;
        if occ & (1u64 << sq_idx) != 0 {
            break;
        }
        f += 1;
    }

    // West
    f = file - 1;
    while f >= EDGE_MIN {
        let sq_idx: usize = (rank as usize) * BOARD_SIZE + (f as usize);
        attacks |= 1u64 << sq_idx;
        if occ & (1u64 << sq_idx) != 0 {
            break;
        }
        f -= 1;
    }

    attacks
}

#[allow(long_running_const_eval)]
pub static ROOK_ATTACKS: [[u64; MAX_ROOK_OCC_VARIATIONS]; NUM_SQUARES] = {
    let mut table = [[EMPTY; MAX_ROOK_OCC_VARIATIONS]; NUM_SQUARES];
    let mut sq = 0;
    while sq < NUM_SQUARES {
        let mask = ROOK_MASKS[sq];
        let occ_variations = 1usize << mask.count_ones();
        let mut index = 0;
        while index < occ_variations {
            let mut occ = EMPTY;
            let mut bits = mask;
            let subset = index;
            let mut bit_index = 0;
            while bits != 0 {
                let lsb = bits & bits.wrapping_neg();
                if (subset >> bit_index) & 1 != 0 {
                    occ |= lsb;
                }
                bits &= bits - 1;
                bit_index += 1;
            }
            table[sq][index] = rook_attacks_from(sq as u8, occ);
            index += 1;
        }
        sq += 1;
    }
    table
};

pub const BISHOP_MASKS: [u64; NUM_SQUARES] = {
    let mut masks = [EMPTY; NUM_SQUARES];
    let mut sq = 0;
    while sq < NUM_SQUARES {
        masks[sq] = bishop_mask(sq as u8);
        sq += 1;
    }
    masks
};

const fn bishop_mask(sq: u8) -> u64 {
    let mut mask = EMPTY;
    let rank: i8 = (sq as usize / BOARD_SIZE) as i8;
    let file: i8 = (sq as usize % BOARD_SIZE) as i8;

    // NE
    let mut r = rank + 1;
    let mut f = file + 1;
    while r <= INNER_MAX && f <= INNER_MAX {
        mask |= bit(((r as usize) * BOARD_SIZE + f as usize) as u8);
        r += 1;
        f += 1;
    }
    // NW
    r = rank + 1;
    f = file - 1;
    while r <= INNER_MAX && f >= INNER_MIN {
        mask |= bit(((r as usize) * BOARD_SIZE + f as usize) as u8);
        r += 1;
        f -= 1;
    }
    // SE
    r = rank - 1;
    f = file + 1;
    while r >= INNER_MIN && f <= INNER_MAX {
        mask |= bit(((r as usize) * BOARD_SIZE + f as usize) as u8);
        r -= 1;
        f += 1;
    }
    // SW
    r = rank - 1;
    f = file - 1;
    while r >= INNER_MIN && f >= INNER_MIN {
        mask |= bit(((r as usize) * BOARD_SIZE + f as usize) as u8);
        r -= 1;
        f -= 1;
    }

    mask
}

const fn bishop_attacks_from(sq: u8, occ: u64) -> u64 {
    let mut attacks = EMPTY;
    let rank: i8 = (sq as usize / BOARD_SIZE) as i8;
    let file: i8 = (sq as usize % BOARD_SIZE) as i8;

    // NE
    let mut r = rank + 1;
    let mut f = file + 1;
    while r <= EDGE_MAX && f <= EDGE_MAX {
        let sq_idx = (r as usize) * BOARD_SIZE + (f as usize);
        attacks |= bit(sq_idx as u8);
        if occ & bit(sq_idx as u8) != 0 {
            break;
        }
        r += 1;
        f += 1;
    }
    // NW
    r = rank + 1;
    f = file - 1;
    while r <= EDGE_MAX && f >= EDGE_MIN {
        let sq_idx = (r as usize) * BOARD_SIZE + (f as usize);
        attacks |= bit(sq_idx as u8);
        if occ & bit(sq_idx as u8) != 0 {
            break;
        }
        r += 1;
        f -= 1;
    }
    // SE
    r = rank - 1;
    f = file + 1;
    while r >= EDGE_MIN && f <= EDGE_MAX {
        let sq_idx = (r as usize) * BOARD_SIZE + (f as usize);
        attacks |= bit(sq_idx as u8);
        if occ & bit(sq_idx as u8) != 0 {
            break;
        }
        r -= 1;
        f += 1;
    }
    // SW
    r = rank - 1;
    f = file - 1;
    while r >= EDGE_MIN && f >= EDGE_MIN {
        let sq_idx = (r as usize) * BOARD_SIZE + (f as usize);
        attacks |= bit(sq_idx as u8);
        if occ & bit(sq_idx as u8) != 0 {
            break;
        }
        r -= 1;
        f -= 1;
    }

    attacks
}

pub static BISHOP_ATTACKS: [[u64; 512]; NUM_SQUARES] = {
    let mut table = [[EMPTY; 512]; NUM_SQUARES];
    let mut sq = 0;
    while sq < NUM_SQUARES {
        let mask = bishop_mask(sq as u8);
        let occ_variations = 1usize << mask.count_ones();
        let mut index = 0;
        while index < occ_variations {
            // Map index bits into actual occupancy bits
            let mut occ = EMPTY;
            let mut bits = mask;
            let subset = index;
            let mut bit_index = 0;
            while bits != 0 {
                let lsb = bits & bits.wrapping_neg();
                if (subset >> bit_index) & 1 != 0 {
                    occ |= lsb;
                }
                bits &= bits - 1;
                bit_index += 1;
            }
            table[sq][index] = bishop_attacks_from(sq as u8, occ);
            index += 1;
        }
        sq += 1;
    }
    table
};

pub const PAWN_ATTACKS: [[u64; 64]; 2] = {
    let mut table = [[0u64; 64]; 2];

    // White pawn attacks
    let mut sq = 0;
    while sq < 64 {
        let bb = 1u64 << sq;
        table[Color::White as usize][sq] = ((bb >> 7) & !FILE_A) | ((bb >> 9) & !FILE_H);
        sq += 1;
    }

    // Black pawn attacks
    let mut sq = 0;
    while sq < 64 {
        let bb = 1u64 << sq;
        table[Color::Black as usize][sq] = ((bb << 7) & !FILE_H) | ((bb << 9) & !FILE_A);
        sq += 1;
    }

    table
};

pub const RANK_MASKS: [u64; NUM_SQUARES] = {
    let mut table = [EMPTY; NUM_SQUARES];
    let mut sq = 0;
    while sq < NUM_SQUARES {
        let rank = sq / BOARD_SIZE;
        let mut mask = EMPTY;
        let mut f = 0;
        while f < BOARD_SIZE {
            mask |= 1u64 << (rank * BOARD_SIZE + f);
            f += 1;
        }
        table[sq] = mask;
        sq += 1;
    }
    table
};

pub const FILE_MASKS: [u64; NUM_SQUARES] = {
    let mut table = [EMPTY; NUM_SQUARES];
    let mut sq = 0;
    while sq < NUM_SQUARES {
        let file = sq % BOARD_SIZE;
        let mut mask = EMPTY;
        let mut r = 0;
        while r < BOARD_SIZE {
            mask |= 1u64 << (r * BOARD_SIZE + file);
            r += 1;
        }
        table[sq] = mask;
        sq += 1;
    }
    table
};

pub const DIAGONAL_MASKS: [u64; NUM_SQUARES] = {
    let mut table = [EMPTY; NUM_SQUARES];
    let mut sq = 0;
    while sq < NUM_SQUARES {
        let rank = sq / BOARD_SIZE;
        let file = sq % BOARD_SIZE;
        let mut mask = EMPTY;

        // Walk NE
        let mut r = rank;
        let mut f = file;
        while r < BOARD_SIZE && f < BOARD_SIZE {
            mask |= 1u64 << (r * BOARD_SIZE + f);
            r += 1;
            f += 1;
        }
        // Walk SW
        let mut r = rank;
        let mut f = file;
        while r < BOARD_SIZE && f < BOARD_SIZE {
            mask |= 1u64 << (r * BOARD_SIZE + f);
            if r == 0 || f == 0 {
                break;
            }
            r -= 1;
            f -= 1;
        }

        table[sq] = mask;
        sq += 1;
    }
    table
};

pub const ANTIDIAGONAL_MASKS: [u64; NUM_SQUARES] = {
    let mut table = [EMPTY; NUM_SQUARES];
    let mut sq = 0;
    while sq < NUM_SQUARES {
        let rank = sq / BOARD_SIZE;
        let file = sq % BOARD_SIZE;
        let mut mask = EMPTY;

        // Walk NW
        let mut r = rank;
        let mut f = file;
        while r < BOARD_SIZE && f < BOARD_SIZE {
            mask |= 1u64 << (r * BOARD_SIZE + f);
            if r == BOARD_SIZE - 1 || f == 0 {
                break;
            }
            r += 1;
            f -= 1;
        }
        // Walk SE
        let mut r = rank;
        let mut f = file;
        while r < BOARD_SIZE && f < BOARD_SIZE {
            mask |= 1u64 << (r * BOARD_SIZE + f);
            if r == 0 || f == BOARD_SIZE - 1 {
                break;
            }
            r -= 1;
            f += 1;
        }

        table[sq] = mask;
        sq += 1;
    }
    table
};
