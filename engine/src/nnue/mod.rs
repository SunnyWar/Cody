// nnue/mod.rs
// Root module for NNUE evaluation

pub mod accumulator;
pub mod features;
pub mod network;
pub mod update;

// Re-export main NNUE types

use bitboard::position::Position;

#[derive(Clone)]
pub struct NNUE {
    pub weights: &'static [u8], // Embedded NNUE weights
}

impl NNUE {
    pub fn new() -> Self {
        static NNUE_WEIGHTS: &[u8] = include_bytes!("../../../NNUE/chak-068cc47e57f2.nnue");
        NNUE {
            weights: NNUE_WEIGHTS,
        }
    }

    pub fn evaluate(&self, pos: &Position) -> i32 {
        let features = crate::nnue::features::extract_piece_features(pos);
        let acc = crate::nnue::accumulator::accumulate_features(&features);
        crate::nnue::network::network_infer(&acc)
    }
}

impl Default for NNUE {
    fn default() -> Self {
        Self::new()
    }
}

use crate::search::evaluator::Evaluator;

#[derive(Clone)]
pub struct NNUEEvaluator {
    pub nnue: NNUE,
}

impl Evaluator for NNUEEvaluator {
    fn evaluate(&self, pos: &Position) -> i32 {
        self.nnue.evaluate(pos)
    }
}

// ...existing code...
