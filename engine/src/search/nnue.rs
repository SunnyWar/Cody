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
        // Minimal stub: sum piece-square indices as a fake feature
        // (Real NNUE would extract features and run through the network)
        let mut sum = 0i32;
        for sq in 0..64 {
            // Use piece_on array to get the piece at each square
            let piece = _pos.piece_on[sq];
            // Only count non-empty squares
            if piece as u8 != 0 {
                sum += (piece as i32) * (sq as i32);
            }
        }
        sum // Placeholder: replace with NNUE inference
    }
}

impl Default for NNUE {
    fn default() -> Self {
        Self::new()
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
