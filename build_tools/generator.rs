// build_tools/generator.rs

use std::fs;
use std::path::Path;

use crate::build_tools::bitboard::files::FileBitboards;
use crate::build_tools::bitboard::ranks::RankBitboards;
use crate::build_tools::bitboard::square_color_mask::SquareColorMask;

pub trait CodeGenerator {
    fn filename(&self) -> &'static str;
    fn generate(&self) -> String;
}

pub fn write_generated_file(out_path: &Path, filename: &str, content: &str) {
    let dest_path = out_path.join(filename);
    fs::write(dest_path, content).expect("Failed to write generated file");
}

pub fn run_generators(out_path: &Path) {
    let generators: Vec<Box<dyn CodeGenerator>> = vec![
        Box::new(RankBitboards),
        Box::new(FileBitboards),
        Box::new(SquareColorMask),
    ];

    for generator in generators {
        let content = generator.generate();
        write_generated_file(out_path, generator.filename(), &content);
    }
}
