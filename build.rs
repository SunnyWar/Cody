use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Path to the generated file
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated_tests.rs");

    // Count the number of test cases by reading src/test_data.rs
    // This is a simple heuristic: count lines with `TestCase {`
    let test_data = fs::read_to_string("src/test_data.rs").unwrap();
    let count = test_data.matches("TestCase {").count();

    // Generate the macro invocation
    let mut output = String::new();
    output.push_str("gen_tests! {\n");
    for i in 0..count {
        output.push_str(&format!("    case_{}: {},\n", i, i));
    }
    output.push_str("}\n");

    // Write the generated file
    fs::write(dest_path, output).unwrap();
}
