// nnue/features.rs
// NNUE feature extraction logic

use bitboard::piece::Piece;
use bitboard::position::Position;
use bitboard::square::Square;

/// Standard NNUE feature extraction: king-square and piece-square features.
/// Returns a vector of (king_square, piece, square) for all pieces.
pub fn extract_nnue_features(pos: &Position) -> Vec<(Square, Piece, Square)> {
    let mut features = Vec::new();
    // Find king squares for both colors
    let mut white_king_sq = None;
    let mut black_king_sq = None;
    for sq in 0..64 {
        let piece = pos.piece_on[sq];
        if piece == Piece::WhiteKing {
            white_king_sq = Some(Square::try_from_index(sq as u8).unwrap());
        } else if piece == Piece::BlackKing {
            black_king_sq = Some(Square::try_from_index(sq as u8).unwrap());
        }
    }
    let white_king_sq = white_king_sq.expect("White king not found");
    let black_king_sq = black_king_sq.expect("Black king not found");

    // For each piece, add (king_square, piece, square) for both perspectives
    for sq in 0..64 {
        let piece = pos.piece_on[sq];
        if piece != Piece::None {
            let square = Square::try_from_index(sq as u8).unwrap();
            // White's perspective
            features.push((white_king_sq, piece, square));
            // Black's perspective
            features.push((black_king_sq, piece, square));
        }
    }
    features
}
