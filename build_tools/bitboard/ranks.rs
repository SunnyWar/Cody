// build_tools/bitboard/ranks.rs
use std::fs;
use std::path::Path;

pub fn generate_rank_bitboards(out_path: &Path) {
    let mut output = String::new();
    output.push_str("// Auto-generated rank bitboards\n\n");

    let mut value: u64 = 0x00000000000000FF;
    let mut rank_names = Vec::new();

    for rank in 1..=8 {
        let const_name = format!("RANK_{}", rank);
        output.push_str(&format!(
            "pub const {}: BitBoardMask = BitBoardMask(0x{:016X});\n",
            const_name, value
        ));
        rank_names.push(const_name);
        value <<= 8;
    }

    output.push('\n');
    output.push_str("pub const RANK_MASKS: [BitBoardMask; BOARD_SIZE] = [\n");
    for name in &rank_names {
        output.push_str(&format!("    {},\n", name));
    }
    output.push_str("];\n");

    let dest_path = out_path.join("generated_ranks.rs");
    fs::write(dest_path, output).expect("Failed to write generated_ranks.rs");
}
