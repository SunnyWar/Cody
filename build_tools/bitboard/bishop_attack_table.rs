use crate::build_tools::{bitboard::rook_attack_table::BitBoardMask, generator::CodeGenerator};

pub struct BishopAttackTable;

impl CodeGenerator for BishopAttackTable {
    fn filename(&self) -> &'static str {
        "generated_bishop_attacks.rs"
    }

    fn generate(&self) -> String {
        const NUM_SQUARES: usize = 64;
        let mut output = String::new();
        output.push_str("// Auto-generated bishop attack table\n\n");
        output.push_str("pub const BISHOP_ATTACKS: [&[BitBoardMask]; 64] = [\n");

        for square in 0..NUM_SQUARES {
            let mask = bishop_relevant_mask(square);
            let num_variations = 1usize << (mask.count_ones() as usize);

            output.push_str("    &[\n");
            for index in 0..num_variations {
                let occupancy = index_to_occupancy(index, mask);
                let attacks = compute_bishop_attacks(square, occupancy);
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

/// Build-helper: same as for rook, reusing index_to_occupancy
fn bishop_relevant_mask(square: usize) -> BitBoardMask {
    let rank = square / 8;
    let file = square % 8;
    let mut mask = 0u64;

    for (dr, df) in &[(1, 1), (1, -1), (-1, 1), (-1, -1)] {
        let mut r = rank as i8 + dr;
        let mut f = file as i8 + df;
        while (0..8).contains(&r) && (0..8).contains(&f) {
            mask |= 1u64 << ((r as usize) * 8 + (f as usize));
            r += dr;
            f += df;
        }
    }

    BitBoardMask(mask)
}

fn compute_bishop_attacks(square: usize, occupancy: BitBoardMask) -> BitBoardMask {
    let rank = square / 8;
    let file = square % 8;
    let mut attacks = 0u64;

    for (dr, df) in &[(1, 1), (1, -1), (-1, 1), (-1, -1)] {
        let mut r = rank as i8 + dr;
        let mut f = file as i8 + df;
        while (0..8).contains(&r) && (0..8).contains(&f) {
            let sq = (r as usize) * 8 + (f as usize);
            attacks |= 1u64 << sq;
            if occupancy.0 & (1u64 << sq) != 0 {
                break;
            }
            r += dr;
            f += df;
        }
    }

    BitBoardMask(attacks)
}
