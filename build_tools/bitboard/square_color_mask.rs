use std::fs;
use std::path::Path;

pub fn generate_square_color_mask(out_path: &Path) {
    const BOARD_SIZE: usize = 8;
    const NUM_SQUARES: usize = BOARD_SIZE * BOARD_SIZE;

    // Compute LIGHT_SQUARES
    let mut light_squares: u64 = 0;
    for sq in 0..NUM_SQUARES {
        let rank = sq / BOARD_SIZE;
        let file = sq % BOARD_SIZE;
        if (rank + file) % 2 == 0 {
            light_squares |= 1u64 << sq;
        }
    }

    let dark_squares: u64 = !light_squares;

    // Compute SQUARE_COLOR_MASK
    let mut mask_array = Vec::new();
    for sq in 0..NUM_SQUARES {
        let rank = sq / BOARD_SIZE;
        let file = sq % BOARD_SIZE;
        let mask = if (rank + file) % 2 == 0 {
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

    let dest_path = out_path.join("generated_square_color.rs");
    fs::write(dest_path, output).expect("Failed to write generated_square_color.rs");
}
