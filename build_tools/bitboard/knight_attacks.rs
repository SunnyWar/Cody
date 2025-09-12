use crate::build_tools::generator::CodeGenerator;

pub struct KnightAttackTable;

impl CodeGenerator for KnightAttackTable {
    fn filename(&self) -> &'static str {
        "generated_knight_attacks.rs"
    }

    fn generate(&self) -> String {
        const BOARD_SIZE: usize = 8;
        const NUM_SQUARES: usize = BOARD_SIZE * BOARD_SIZE;

        let mut output = String::new();
        output.push_str("// Auto-generated knight attack table\n\n");
        output.push_str("pub const KNIGHT_ATTACKS: [BitBoardMask; 64] = [\n");

        for square in 0..NUM_SQUARES {
            let rank = square / BOARD_SIZE;
            let file = square % BOARD_SIZE;
            let mut attack_mask: u64 = 0;

            for (dr, df) in [
                (2, 1),
                (1, 2),
                (-1, 2),
                (-2, 1),
                (-2, -1),
                (-1, -2),
                (1, -2),
                (2, -1),
            ] {
                let r = rank as i8 + dr;
                let f = file as i8 + df;
                if (0..8).contains(&r) && (0..8).contains(&f) {
                    let target = (r as usize) * 8 + (f as usize);
                    attack_mask |= 1u64 << target;
                }
            }

            output.push_str(&format!("    BitBoardMask(0x{:016X}),\n", attack_mask));
        }

        output.push_str("];\n");
        output
    }
}
