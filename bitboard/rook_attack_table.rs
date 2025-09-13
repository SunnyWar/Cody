use crate::build_tools::generator::CodeGenerator;

#[derive(Copy, Clone)]
pub struct BitBoardMask(pub u64);

impl BitBoardMask {
    #[inline]
    pub fn count_ones(self) -> u32 {
        self.0.count_ones()
    }
}

pub struct RookAttackTable;

impl CodeGenerator for RookAttackTable {
    fn filename(&self) -> &'static str {
        "generated_rook_attacks.rs"
    }

    fn generate(&self) -> String {
        const NUM_SQUARES: usize = 64;
        let mut output = String::new();
        output.push_str("// Auto-generated rook attack table\n\n");
        output.push_str("pub const ROOK_ATTACKS: [&[BitBoardMask]; 64] = [\n");

        for square in 0..NUM_SQUARES {
            let mask = rook_relevant_mask(square);
            let num_variations = 1usize << (mask.count_ones() as usize);

            output.push_str("    &[\n");
            for index in 0..num_variations {
                let occupancy = index_to_occupancy(index, mask);
                let attacks = compute_rook_attacks(square, occupancy);
                output.push_str(&format!("        BitBoardMask(0x{:016X}),\n", attacks.0));
            }
            output.push_str("    ],\n");
        }

        output.push_str("];\n");
        output
    }
}

fn index_to_occupancy(index: usize, mask: BitBoardMask) -> BitBoardMask {
    let mut occupancy = 0u64;
    let mut bits = mask.0;
    let mut bit_pos = 0;

    while bits != 0 {
        let lsb = bits & bits.wrapping_neg();
        if (index >> bit_pos) & 1 != 0 {
            occupancy |= lsb;
        }
        bits &= bits - 1;
        bit_pos += 1;
    }

    BitBoardMask(occupancy)
}

fn rook_relevant_mask(square: usize) -> BitBoardMask {
    let rank = square / 8;
    let file = square % 8;

    let mut mask = 0u64;

    // Horizontal (rank) excluding origin
    for f in 0..8 {
        if f != file {
            mask |= 1u64 << (rank * 8 + f);
        }
    }

    // Vertical (file) excluding origin
    for r in 0..8 {
        if r != rank {
            mask |= 1u64 << (r * 8 + file);
        }
    }

    BitBoardMask(mask)
}

fn compute_rook_attacks(square: usize, occupancy: BitBoardMask) -> BitBoardMask {
    let rank = square / 8;
    let file = square % 8;
    let mut attacks = 0u64;

    // Horizontal (left)
    for f in (0..file).rev() {
        let sq = rank * 8 + f;
        attacks |= 1u64 << sq;
        if occupancy.0 & (1u64 << sq) != 0 {
            break;
        }
    }

    // Horizontal (right)
    for f in (file + 1)..8 {
        let sq = rank * 8 + f;
        attacks |= 1u64 << sq;
        if occupancy.0 & (1u64 << sq) != 0 {
            break;
        }
    }

    // Vertical (up)
    for r in (0..rank).rev() {
        let sq = r * 8 + file;
        attacks |= 1u64 << sq;
        if occupancy.0 & (1u64 << sq) != 0 {
            break;
        }
    }

    // Vertical (down)
    for r in (rank + 1)..8 {
        let sq = r * 8 + file;
        attacks |= 1u64 << sq;
        if occupancy.0 & (1u64 << sq) != 0 {
            break;
        }
    }

    BitBoardMask(attacks)
}
