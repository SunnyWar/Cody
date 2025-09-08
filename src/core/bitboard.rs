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

pub const LIGHT_SQUARES: u64 = {
    let mut mask = 0u64;
    let mut sq = 0;
    while sq < 64 {
        let rank = sq / 8;
        let file = sq % 8;
        if (rank + file) % 2 == 0 {
            mask |= 1u64 << sq;
        }
        sq += 1;
    }
    mask
};

pub const DARK_SQUARES: u64 = !LIGHT_SQUARES;

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

const fn gen_king_attacks(square: usize) -> u64 {
    let mut attacks = 0u64;
    let rank = square / 8;
    let file = square % 8;

    let mut r = rank as i32 - 1;
    while r <= rank as i32 + 1 {
        let mut f = file as i32 - 1;
        while f <= file as i32 + 1 {
            if r >= 0 && r < 8 && f >= 0 && f < 8 {
                let sq = (r * 8 + f) as usize;
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

const fn init_king_attacks() -> [u64; 64] {
    let mut table = [0u64; 64];
    let mut sq = 0;
    while sq < 64 {
        table[sq] = gen_king_attacks(sq);
        sq += 1;
    }
    table
}

pub const KING_ATTACKS: [u64; 64] = init_king_attacks();

const fn knight_attacks_for(sq: u8) -> u64 {
    let b = bit(sq);
    let l1 = (b >> 1) & 0x7f7f7f7f7f7f7f7f;
    let l2 = (b >> 2) & 0x3f3f3f3f3f3f3f3f;
    let r1 = (b << 1) & 0xfefefefefefefefe;
    let r2 = (b << 2) & 0xfcfcfcfcfcfcfcfc;
    let h1 = l1 | r1;
    let h2 = l2 | r2;
    (h1 << 16) | (h1 >> 16) | (h2 << 8) | (h2 >> 8)
}

pub const KNIGHT_ATTACKS: [u64; 64] = {
    let mut table = [0u64; 64];
    let mut sq = 0;
    while sq < 64 {
        table[sq] = knight_attacks_for(sq as u8);
        sq += 1;
    }
    table
};

const fn rook_mask(sq: u8) -> u64 {
    let mut mask = 0u64;
    let rank = (sq / 8) as i8;
    let file = (sq % 8) as i8;

    // North
    let mut r = rank + 1;
    while r <= 6 {
        mask |= 1u64 << (r * 8 + file);
        r += 1;
    }
    // South
    r = rank - 1;
    while r >= 1 {
        mask |= 1u64 << (r * 8 + file);
        r -= 1;
    }
    // East
    let mut f = file + 1;
    while f <= 6 {
        mask |= 1u64 << (rank * 8 + f);
        f += 1;
    }
    // West
    f = file - 1;
    while f >= 1 {
        mask |= 1u64 << (rank * 8 + f);
        f -= 1;
    }

    mask
}

pub const ROOK_MASKS: [u64; 64] = {
    let mut table = [0u64; 64];
    let mut sq = 0;
    while sq < 64 {
        table[sq] = rook_mask(sq as u8);
        sq += 1;
    }
    table
};

const fn rook_attacks_from(sq: u8, occ: u64) -> u64 {
    let mut attacks = 0u64;
    let rank = (sq / 8) as i8;
    let file = (sq % 8) as i8;

    // North
    let mut r = rank + 1;
    while r <= 7 {
        let sq_idx = (r * 8 + file) as u8;
        attacks |= 1u64 << sq_idx;
        if occ & (1u64 << sq_idx) != 0 {
            break;
        }
        r += 1;
    }
    // South
    r = rank - 1;
    while r >= 0 {
        let sq_idx = (r * 8 + file) as u8;
        attacks |= 1u64 << sq_idx;
        if occ & (1u64 << sq_idx) != 0 {
            break;
        }
        r -= 1;
    }
    // East
    let mut f = file + 1;
    while f <= 7 {
        let sq_idx = (rank * 8 + f) as u8;
        attacks |= 1u64 << sq_idx;
        if occ & (1u64 << sq_idx) != 0 {
            break;
        }
        f += 1;
    }
    // West
    f = file - 1;
    while f >= 0 {
        let sq_idx = (rank * 8 + f) as u8;
        attacks |= 1u64 << sq_idx;
        if occ & (1u64 << sq_idx) != 0 {
            break;
        }
        f -= 1;
    }

    attacks
}

#[allow(long_running_const_eval)]
pub static ROOK_ATTACKS: [[u64; 4096]; 64] = {
    let mut table = [[0u64; 4096]; 64];
    let mut sq = 0;
    while sq < 64 {
        let mask = ROOK_MASKS[sq];
        let occ_variations = 1usize << mask.count_ones();
        let mut index = 0;
        while index < occ_variations {
            // Build occupancy from index bits
            let mut occ = 0u64;
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

pub const BISHOP_MASKS: [u64; 64] = {
    let mut masks = [0u64; 64];
    let mut sq = 0;
    while sq < 64 {
        masks[sq] = bishop_mask(sq as u8);
        sq += 1;
    }
    masks
};

const fn bishop_mask(sq: u8) -> u64 {
    let mut mask = 0u64;
    let rank = (sq / 8) as i8;
    let file = (sq % 8) as i8;

    // NE
    let mut r = rank + 1;
    let mut f = file + 1;
    while r <= 6 && f <= 6 {
        mask |= bit((r * 8 + f) as u8);
        r += 1;
        f += 1;
    }
    // NW
    r = rank + 1;
    f = file - 1;
    while r <= 6 && f >= 1 {
        mask |= bit((r * 8 + f) as u8);
        r += 1;
        f -= 1;
    }
    // SE
    r = rank - 1;
    f = file + 1;
    while r >= 1 && f <= 6 {
        mask |= bit((r * 8 + f) as u8);
        r -= 1;
        f += 1;
    }
    // SW
    r = rank - 1;
    f = file - 1;
    while r >= 1 && f >= 1 {
        mask |= bit((r * 8 + f) as u8);
        r -= 1;
        f -= 1;
    }

    mask
}

const fn bishop_attacks_from(sq: u8, occ: u64) -> u64 {
    let mut attacks = 0u64;
    let rank = (sq / 8) as i8;
    let file = (sq % 8) as i8;

    // NE
    let mut r = rank + 1;
    let mut f = file + 1;
    while r <= 7 && f <= 7 {
        let sq_idx = (r * 8 + f) as u8;
        attacks |= bit(sq_idx);
        if occ & bit(sq_idx) != 0 {
            break;
        }
        r += 1;
        f += 1;
    }
    // NW
    r = rank + 1;
    f = file - 1;
    while r <= 7 && f >= 0 {
        let sq_idx = (r * 8 + f) as u8;
        attacks |= bit(sq_idx);
        if occ & bit(sq_idx) != 0 {
            break;
        }
        r += 1;
        f -= 1;
    }
    // SE
    r = rank - 1;
    f = file + 1;
    while r >= 0 && f <= 7 {
        let sq_idx = (r * 8 + f) as u8;
        attacks |= bit(sq_idx);
        if occ & bit(sq_idx) != 0 {
            break;
        }
        r -= 1;
        f += 1;
    }
    // SW
    r = rank - 1;
    f = file - 1;
    while r >= 0 && f >= 0 {
        let sq_idx = (r * 8 + f) as u8;
        attacks |= bit(sq_idx);
        if occ & bit(sq_idx) != 0 {
            break;
        }
        r -= 1;
        f -= 1;
    }

    attacks
}

pub static BISHOP_ATTACKS: [[u64; 512]; 64] = {
    let mut table = [[0u64; 512]; 64];
    let mut sq = 0;
    while sq < 64 {
        let mask = bishop_mask(sq as u8);
        let occ_variations = 1u64 << mask.count_ones();
        let mut index = 0;
        while index < occ_variations {
            // Map index bits into actual occupancy bits
            let mut occ = 0u64;
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
            table[sq][index as usize] = bishop_attacks_from(sq as u8, occ);
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
