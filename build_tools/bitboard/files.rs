// build_tools/bitboard/files.rs
use std::fs;
use std::path::Path;

pub fn generate_file_bitboards(out_path: &Path) {
    let mut output = String::new();
    output.push_str("// Auto-generated file bitboards\n\n");

    let mut value: u64 = 0x0101010101010101;
    let mut file_masks = Vec::new();
    let mut file_names = Vec::new();

    for file in ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'] {
        let const_name = format!("FILE_{}", file);
        output.push_str(&format!(
            "pub const {}: BitBoardMask = BitBoardMask(0x{:016X});\n",
            const_name, value
        ));
        file_names.push(const_name.clone());
        file_masks.push(value);
        value <<= 1;
    }

    output.push('\n');
    output.push_str("pub const FILE_MASKS: [BitBoardMask; BOARD_SIZE] = [\n");
    for name in &file_names {
        output.push_str(&format!("    {},\n", name));
    }
    output.push_str("];\n");

    output.push('\n');
    output.push_str("// Masks excluding specific files\n");

    let not_file_a = !file_masks[0];
    let not_file_ab = !(file_masks[0] | file_masks[1]);
    let not_file_h = !file_masks[7];
    let not_file_gh = !(file_masks[6] | file_masks[7]);

    output.push_str(&format!(
        "pub const NOT_FILE_A: BitBoardMask = BitBoardMask(0x{:016X});\n",
        not_file_a
    ));
    output.push_str(&format!(
        "pub const NOT_FILE_AB: BitBoardMask = BitBoardMask(0x{:016X});\n",
        not_file_ab
    ));
    output.push_str(&format!(
        "pub const NOT_FILE_H: BitBoardMask = BitBoardMask(0x{:016X});\n",
        not_file_h
    ));
    output.push_str(&format!(
        "pub const NOT_FILE_GH: BitBoardMask = BitBoardMask(0x{:016X});\n",
        not_file_gh
    ));

    let dest_path = out_path.join("generated_files.rs");
    fs::write(dest_path, output).expect("Failed to write generated_files.rs");
}
