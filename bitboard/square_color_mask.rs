// build_tools/bitboard/square_color_mask.rs

use crate::build_tools::generator::CodeGenerator;
pub struct SquareColorMask;

impl CodeGenerator for SquareColorMask {
    fn filename(&self) -> &'static str {
        "generated_square_color.rs"
    }

    fn generate(&self) -> String {
        const BOARD_SIZE: usize = 8;
        const NUM_SQUARES: usize = BOARD_SIZE * BOARD_SIZE;

        // Compute LIGHT_SQUARES
        let mut light_squares: u64 = 0;
        for sq in 0..NUM_SQUARES {
            let rank = sq / BOARD_SIZE;
            let file = sq % BOARD_SIZE;
            if (rank + file).is_multiple_of(2) {
                light_squares |= 1u64 << sq;
            }
        }

        let dark_squares: u64 = !light_squares;

        // Compute SQUARE_COLOR_MASK
        let mut mask_array = Vec::new();
        for sq in 0..NUM_SQUARES {
            let rank = sq / BOARD_SIZE;
            let file = sq % BOARD_SIZE;
            let mask = if (rank + file).is_multiple_of(2) {
                light_squares
            } else {
                dark_squares
            };
            mask_array.push(mask);
        }

        // Generate Rust source
        let mut output = String::new();
        output.push_str("// Auto-generated square color mask\n\n");
        output.push_str("pub const SQUARE_COLOR_MASK: [u64; 64] = [\n");
        for mask in mask_array {
            output.push_str(&format!("    0x{:016X},\n", mask));
        }
        output.push_str("];\n");

        output
    }
}
