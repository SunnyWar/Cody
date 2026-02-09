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

    // Check for perft command
    if args.is_empty() || (args.len() > 0 && args[0] != "perft") {
        // UCI mode or handle flags
        for a in args.iter() {
            if a == "--verbose" || a.eq_ignore_ascii_case("-v") {
                VERBOSE.store(true, std::sync::atomic::Ordering::Relaxed);
            }
        }

        let mut api = api::uciapi::CodyApi::new();
        api.run();
    } else {
        // Perft mode
        let depth = if args.len() > 1 {
            args[1].parse::<u32>().unwrap_or(5)
        } else {
            5
        };

        let pos = Position::default();
        util::run_perft_benchmark(&pos, depth);
    }
}
