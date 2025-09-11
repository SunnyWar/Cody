// build_tools/bitboard/files.rs
use std::fs;
use std::path::Path;

pub fn generate_file_bitboards(out_path: &Path) {
    let mut output = String::new();
    output.push_str("// Auto-generated file bitboards\n\n");

    let mut value: u64 = 0x0101010101010101;
    let mut file_names = Vec::new();

    for file in ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'] {
        let const_name = format!("FILE_{}", file);
        let visibility = if file == 'A' || file == 'H' {
            "pub "
        } else {
            ""
        };
        output.push_str(&format!(
            "{}const {}: BitBoardMask = BitBoardMask(0x{:016X});\n",
            visibility, const_name, value
        ));
        file_names.push(const_name);
        value <<= 1;
    }

    output.push('\n');
    output.push_str("pub const FILE_MASKS: [BitBoardMask; BOARD_SIZE] = [\n");
    for name in &file_names {
        output.push_str(&format!("    {},\n", name));
    }
    output.push_str("];\n");

    let dest_path = out_path.join("generated_files.rs");
    fs::write(dest_path, output).expect("Failed to write generated_files.rs");
}
