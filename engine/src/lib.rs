// src/lib.rs
pub mod core;
pub mod search;
pub mod test_data;

pub use crate::search::engine::{Engine, NODE_COUNT};
pub use crate::search::evaluator::MaterialEvaluator;
pub use crate::test_data::{TEST_CASES, TestCase};
