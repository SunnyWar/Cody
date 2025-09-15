// src/search/evaluator.rs

use bitboard::{
    piece::{Color, Piece, PieceKind},
    position::Position,
};

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

        // White pieces
        for kind in [
            PieceKind::Pawn,
            PieceKind::Knight,
            PieceKind::Bishop,
            PieceKind::Rook,
            PieceKind::Queen,
            PieceKind::King,
        ] {
            let piece = Piece::from_parts(Color::White, Some(kind));
            let bb = pos.pieces.get(piece);
            score += PIECE_VALUES[kind as usize] * bb.0.count_ones() as i32;
        }

        // Black pieces
        for kind in [
            PieceKind::Pawn,
            PieceKind::Knight,
            PieceKind::Bishop,
            PieceKind::Rook,
            PieceKind::Queen,
            PieceKind::King,
        ] {
            let piece = Piece::from_parts(Color::Black, Some(kind));
            let bb = pos.pieces.get(piece);
            score -= PIECE_VALUES[kind as usize] * bb.0.count_ones() as i32;
        }

        score
    }
}
