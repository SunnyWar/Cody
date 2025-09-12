// build.rs
use regex::Regex;
use std::fs;
use std::path::Path;
mod build_tools;

use build_tools::generator::run_generators;

fn main() {
    println!("cargo:rerun-if-changed=build_tools/bitboard");
    println!("cargo:rerun-if-changed=src/test_data.rs");
    println!("cargo:rerun-if-changed=build_tools/bitboard");

    let generated_test_dir = Path::new("tests");

    println!("Generating test cases");
    generate_test_cases(generated_test_dir);
    println!("Generated tests saved to {:?}", generated_test_dir);

    let generated_dir = Path::new("src/generated");
    run_generators(generated_dir);
}

// TODO - move this to \build_tools\tests
fn generate_test_cases(out_path: &Path) {
    let test_data = fs::read_to_string("src/test_data.rs").expect("Failed to read test_data.rs");
    let re = Regex::new(r#"name:\s*"([^"]+)""#).expect("Failed to compile regex");

    let mut output = String::new();
    output.push_str("use cody::{Engine, MaterialEvaluator, SimpleMoveGen, TEST_CASES};\n\n");

    for (i, cap) in re.captures_iter(&test_data).enumerate() {
        let raw_name = &cap[1];
        let sanitized_name = sanitize_name(raw_name);

        output.push_str(&format!(
            "#[test]\n\
             fn {}() {{\n\
                 let case = &TEST_CASES[{}];\n\
                 let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);\n\
                 let (_, score) = engine.search(&case.position(), 2);\n\
                 assert_eq!(\n\
                     score,\n\
                     case.expected_score,\n\
                     \"Test: {{}}\\nFEN: {{}}\\nExpected Score: {{}}\\nActual Score: {{}}\",\n\
                     case.name,\n\
                     case.fen,\n\
                     case.expected_score,\n\
                     score\n\
                 );\n\
             }}\n\n",
            sanitized_name, i
        ));
    }

    let dest_path = out_path.join("generated_tests.rs");
    fs::write(dest_path, output).expect("Failed to write generated_tests.rs");
}

fn sanitize_name(name: &str) -> String {
    name.to_lowercase()
        .replace(|c: char| !c.is_ascii_alphanumeric(), "_")
}
