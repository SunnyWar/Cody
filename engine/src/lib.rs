// src/lib.rs
pub mod core;
pub mod search;
pub mod test_data;
pub mod util;

#[cfg(test)]
mod perft_integration_test;

pub use crate::search::engine::{Engine, NODE_COUNT};
pub use crate::search::evaluator::MaterialEvaluator;
pub use crate::test_data::{TEST_CASES, TestCase};

// Global runtime verbose flag. Can be toggled via UCI setoption or command-line --verbose.
use std::sync::atomic::AtomicBool;
// Removed unused Ordering import
pub static VERBOSE: AtomicBool = AtomicBool::new(false);
