// nnue/update.rs
// NNUE update logic

use super::accumulator::Accumulator;
use bitboard::piece::Piece;
use bitboard::square::Square;

/// NNUE update stub
pub fn update_accumulator(acc: &mut Accumulator, features: &[(Piece, Square)]) {
    acc.sum = 0;
    for (piece, square) in features.iter() {
        acc.sum += (*piece as i32) * (square.index() as i32);
    }
}
