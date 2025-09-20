// src/search/evaluator.rs

use bitboard::{
    piece::{Color, Piece, PieceKind},
    position::Position,
};

use crate::search::piecesquaretable::{
    BISHOP_SQUARE_TABLE, KNIGHT_SQUARE_TABLE, PAWN_SQUARE_TABLE, QUEEN_SQUARE_TABLE,
    ROOK_SQUARE_TABLE,
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

        for &color in &[Color::White, Color::Black] {
            let sign = if color == Color::White { 1 } else { -1 };

            for kind in [
                PieceKind::Pawn,
                PieceKind::Knight,
                PieceKind::Bishop,
                PieceKind::Rook,
                PieceKind::Queen,
                PieceKind::King,
            ] {
                let piece = Piece::from_parts(color, Some(kind));
                let bb = pos.pieces.get(piece);

                for sq in bb.squares() {
                    let idx = if color == Color::White {
                        sq.index()
                    } else {
                        63 - sq.index() // flip for black
                    };

                    let pst_bonus = match kind {
                        PieceKind::Pawn => PAWN_SQUARE_TABLE[idx],
                        PieceKind::Knight => KNIGHT_SQUARE_TABLE[idx],
                        PieceKind::Bishop => BISHOP_SQUARE_TABLE[idx],
                        PieceKind::Rook => ROOK_SQUARE_TABLE[idx],
                        PieceKind::Queen => QUEEN_SQUARE_TABLE[idx],
                        PieceKind::King => KNIGHT_SQUARE_TABLE[idx],
                    };

                    score += sign * (PIECE_VALUES[kind as usize] + pst_bonus);
                }
            }
        }

        score
    }
}
