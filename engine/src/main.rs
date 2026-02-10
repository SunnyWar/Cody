// src/main.rs
#![allow(non_snake_case)]

mod api;

use bitboard::position::Position;
use engine::VERBOSE;
use engine::util;
use std::env;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    // Skip program name
    args.remove(0);

    // If first argument is "perft", run perft mode. Otherwise, run UCI mode
    // (default if no args).
    if !args.is_empty() && args[0] == "perft" {
        // Perft mode
        let depth = if args.len() > 1 {
            args[1].parse::<u32>().unwrap_or(5)
        } else {
            5
        };

        let pos = Position::default();
        util::run_perft_benchmark(&pos, depth);
    } else {
        // UCI mode or handle flags
        for a in &args {
            if a == "--verbose" || a.eq_ignore_ascii_case("-v") {
                VERBOSE.store(true, std::sync::atomic::Ordering::Relaxed);
            }
        }

        let mut api = api::uciapi::CodyApi::new();
        api.run();
    }
}
