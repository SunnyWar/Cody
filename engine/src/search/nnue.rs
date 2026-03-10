// nnue.rs
// NNUE evaluation module for Cody engine
// This file will define the NNUE struct, loading, and inference API.

use bitboard::position::Position;

/// NNUE evaluator struct
#[derive(Clone)]
pub struct NNUE {
    pub weights: &'static [u8], // Embedded NNUE weights
}

impl NNUE {
    /// Initialize NNUE from embedded weights
    pub fn new() -> Self {
        // Embed NNUE weights using include_bytes!
        // Replace the file path with the actual NNUE file path relative to src
        static NNUE_WEIGHTS: &[u8] = include_bytes!("../../../NNUE/chak-068cc47e57f2.nnue");
        NNUE {
            weights: NNUE_WEIGHTS,
        }
    }

    /// Evaluate a position using NNUE
    pub fn evaluate(&self, _pos: &Position) -> i32 {
        // TODO: Implement feature extraction and inference using self.weights
        0 // Placeholder
    }
}

// Use the Evaluator trait from evaluator.rs
use crate::search::evaluator::Evaluator;

/// NNUE evaluator implementation
#[derive(Clone)]
pub struct NNUEEvaluator {
    pub nnue: NNUE,
}

impl Evaluator for NNUEEvaluator {
    fn evaluate(&self, pos: &Position) -> i32 {
        self.nnue.evaluate(pos)
    }
}
