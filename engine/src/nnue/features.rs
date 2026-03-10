// nnue/features.rs
// NNUE feature extraction logic

use bitboard::piece::Piece;
use bitboard::position::Position;
use bitboard::square::Square;

/// Extracts piece features for NNUE from a position.
/// Returns a vector of (Piece, Square) for all non-empty squares.
pub fn extract_piece_features(pos: &Position) -> Vec<(Piece, Square)> {
    let mut features = Vec::new();
    for sq in 0..64 {
        let piece = pos.piece_on[sq];
        if piece as u8 != 0 {
            features.push((piece, Square::try_from_index(sq as u8).unwrap()));
        }
    }
    features
}
