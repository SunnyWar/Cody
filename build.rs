use std::env;
use std::fs;
use std::path::Path;
use regex::Regex;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated_tests.rs");

    let test_data = fs::read_to_string("src/test_data.rs").unwrap();

    // Regex to capture each TestCase block and extract the name
    let re = Regex::new(r#"name:\s*"([^"]+)""#).unwrap();

    let mut output = String::new();
    output.push_str("gen_tests! {\n");

    for (i, cap) in re.captures_iter(&test_data).enumerate() {
        let raw_name = &cap[1];
        let sanitized_name = raw_name
            .to_lowercase()
            .replace(|c: char| !c.is_ascii_alphanumeric(), "_");

        output.push_str(&format!("    {}: {},\n", sanitized_name, i));
    }

    output.push_str("}\n");
    fs::write(dest_path, output).unwrap();
}
