// src/search/evaluator.rs

use bitboard::{
    piece::{Color, Piece, PieceKind},
    position::Position,
};

use crate::search::piecesquaretable::{
    BISHOP_ENDGAME_TABLE, BISHOP_SQUARE_TABLE, KING_ENDGAME_TABLE, KING_MIDGAME_SQUARE_TABLE,
    KNIGHT_ENDGAME_TABLE, KNIGHT_SQUARE_TABLE, MAX_PHASE, PAWN_ENDGAME_TABLE, PAWN_SQUARE_TABLE,
    PHASE_WEIGHTS, QUEEN_ENDGAME_TABLE, QUEEN_SQUARE_TABLE, ROOK_ENDGAME_TABLE, ROOK_SQUARE_TABLE,
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
        let phase = compute_phase(pos);

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
                        63 - sq.index()
                    };

                    let pst_bonus = match kind {
                        PieceKind::Pawn => {
                            blend_tables(PAWN_SQUARE_TABLE[idx], PAWN_ENDGAME_TABLE[idx], phase)
                        }
                        PieceKind::Knight => {
                            blend_tables(KNIGHT_SQUARE_TABLE[idx], KNIGHT_ENDGAME_TABLE[idx], phase)
                        }
                        PieceKind::Bishop => {
                            blend_tables(BISHOP_SQUARE_TABLE[idx], BISHOP_ENDGAME_TABLE[idx], phase)
                        }
                        PieceKind::Rook => {
                            blend_tables(ROOK_SQUARE_TABLE[idx], ROOK_ENDGAME_TABLE[idx], phase)
                        }
                        PieceKind::Queen => {
                            blend_tables(QUEEN_SQUARE_TABLE[idx], QUEEN_ENDGAME_TABLE[idx], phase)
                        }
                        PieceKind::King => blend_tables(
                            KING_MIDGAME_SQUARE_TABLE[idx],
                            KING_ENDGAME_TABLE[idx],
                            phase,
                        ),
                    };

                    score += sign * (PIECE_VALUES[kind as usize] + pst_bonus);
                }
            }
        }

        score
    }
}

fn compute_phase(pos: &Position) -> i32 {
    use PieceKind::*;

    let mut phase = MAX_PHASE;

    for color in [Color::White, Color::Black] {
        // Unrolled for maximum performance
        let pawn_count = pos
            .pieces
            .get(Piece::from_parts(color, Some(Pawn)))
            .0
            .count_ones() as i32;
        let knight_count = pos
            .pieces
            .get(Piece::from_parts(color, Some(Knight)))
            .0
            .count_ones() as i32;
        let bishop_count = pos
            .pieces
            .get(Piece::from_parts(color, Some(Bishop)))
            .0
            .count_ones() as i32;
        let rook_count = pos
            .pieces
            .get(Piece::from_parts(color, Some(Rook)))
            .0
            .count_ones() as i32;
        let queen_count = pos
            .pieces
            .get(Piece::from_parts(color, Some(Queen)))
            .0
            .count_ones() as i32;
        let king_count = pos
            .pieces
            .get(Piece::from_parts(color, Some(King)))
            .0
            .count_ones() as i32;

        phase -= PHASE_WEIGHTS[0] * pawn_count
            + PHASE_WEIGHTS[1] * knight_count
            + PHASE_WEIGHTS[2] * bishop_count
            + PHASE_WEIGHTS[3] * rook_count
            + PHASE_WEIGHTS[4] * queen_count
            + PHASE_WEIGHTS[5] * king_count;
    }

    phase.clamp(0, MAX_PHASE)
}

fn blend_tables(mid: i32, end: i32, phase: i32) -> i32 {
    ((mid * phase) + (end * (MAX_PHASE - phase))) / MAX_PHASE
}
