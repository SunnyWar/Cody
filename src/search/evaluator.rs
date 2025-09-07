// src/search/evaluator.rs
use crate::core::position::Position;

/// Piece values in centipawns
const PIECE_VALUES: [i32; 6] = [
    100, // Pawn
    320, // Knight
    330, // Bishop
    500, // Rook
    900, // Queen
    0,   // King (not scored in material)
];

/// Simple material-count evaluator.
/// Positive = advantage for White, negative = advantage for Black.
pub struct MaterialEvaluator;

pub trait Evaluator {
    fn evaluate(&self, pos: &Position) -> i32;
}

impl Evaluator for MaterialEvaluator {
    fn evaluate(&self, pos: &Position) -> i32 {
        let mut score = 0;

        // White pieces: indices 0..6
        for (i, bb) in pos.pieces[0..6].iter().enumerate() {
            score += PIECE_VALUES[i] * bb.count_ones() as i32;
        }

        // Black pieces: indices 6..12
        for (i, bb) in pos.pieces[6..12].iter().enumerate() {
            score -= PIECE_VALUES[i] * bb.count_ones() as i32;
        }

        score
    }
}
