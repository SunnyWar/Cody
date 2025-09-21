// src/main.rs
#![allow(non_snake_case)]

mod api;

use engine::VERBOSE;
use std::env;

fn main() {
    // Simple CLI flag: --verbose enables runtime verbose logging (same as UCI setoption Verbose true)
    let mut args = env::args();
    // Skip program name
    args.next();
    for a in args {
        if a == "--verbose" || a.eq_ignore_ascii_case("-v") {
            VERBOSE.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }

    let mut api = api::uciapi::CodyApi::new();
    api.run();
}
