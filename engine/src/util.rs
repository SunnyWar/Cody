use bitboard::{perft, position::Position};
use std::time::SystemTime;

/// Return an ISO-8601 / RFC3339 UTC timestamp with millisecond precision.
/// Example: 2025-09-21T14:33:01.123Z
pub fn iso_stamp_ms() -> String {
    humantime::format_rfc3339_millis(SystemTime::now()).to_string()
}

/// Run perft benchmark and print results with timing.
/// Useful for verifying move generation correctness and measuring performance.
pub fn run_perft_benchmark(pos: &Position, depth: u32) {
    let start = SystemTime::now();
    let count = perft(pos, depth);
    let elapsed = start.elapsed().unwrap_or_default();

    println!(
        "perft({}) = {} ({:.3}s)",
        depth,
        count,
        elapsed.as_secs_f64()
    );
}

/// Run perft with divide (one line per move).
pub fn run_perft_divide(pos: &Position, depth: u32) {
    if depth == 0 {
        println!("Divide not meaningful at depth 0");
        return;
    }

    let start = SystemTime::now();
    let output = perft::perft_divide(pos, depth);
    let elapsed = start.elapsed().unwrap_or_default();

    println!("{}", output);
    println!("({:.3}s)", elapsed.as_secs_f64());
}
