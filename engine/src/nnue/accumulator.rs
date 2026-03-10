// nnue/accumulator.rs
// NNUE accumulator logic

use bitboard::piece::Piece;
use bitboard::square::Square;

/// NNUE accumulator stub
pub struct Accumulator {
    pub sum: i32,
}

/// Accumulate features (stub: sum piece and square indices)
pub fn accumulate_features(features: &[(Piece, Square)]) -> Accumulator {
    let mut sum = 0i32;
    for (piece, square) in features.iter() {
        sum += (*piece as i32) * (square.index() as i32);
    }
    Accumulator { sum }
}
