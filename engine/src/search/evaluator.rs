// src/search/evaluator.rs

use crate::search::piecesquaretable::BISHOP_ENDGAME_TABLE;
use crate::search::piecesquaretable::BISHOP_SQUARE_TABLE;
use crate::search::piecesquaretable::KING_ENDGAME_TABLE;
use crate::search::piecesquaretable::KING_MIDGAME_SQUARE_TABLE;
use crate::search::piecesquaretable::KNIGHT_ENDGAME_TABLE;
use crate::search::piecesquaretable::KNIGHT_SQUARE_TABLE;
use crate::search::piecesquaretable::MAX_PHASE;
use crate::search::piecesquaretable::PAWN_ENDGAME_TABLE;
use crate::search::piecesquaretable::PAWN_SQUARE_TABLE;
use crate::search::piecesquaretable::PHASE_WEIGHTS;
use crate::search::piecesquaretable::QUEEN_ENDGAME_TABLE;
use crate::search::piecesquaretable::QUEEN_SQUARE_TABLE;
use crate::search::piecesquaretable::ROOK_ENDGAME_TABLE;
use crate::search::piecesquaretable::ROOK_SQUARE_TABLE;
use bitboard::piece::Color;
use bitboard::piece::Piece;
use bitboard::piece::PieceKind;
use bitboard::position::Position;

/// Piece values in centipawns
const PIECE_VALUES: [i32; 6] = [
    100, // Pawn
    320, // Knight
    330, // Bishop
    500, // Rook
    900, // Queen
    0,   // King (not scored in material)
];

const BISHOP_PAIR_BONUS: i32 = 30;
const DOUBLED_PAWN_PENALTY: i32 = 12;
const ISOLATED_PAWN_PENALTY: i32 = 10;
const PASSED_PAWN_BONUS_BY_ADVANCE: [i32; 8] = [0, 5, 10, 18, 28, 42, 60, 0];

/// Simple material-count evaluator.
/// Positive = advantage for White, negative = advantage for Black.
#[derive(Clone, Copy)]
pub struct MaterialEvaluator;

pub trait Evaluator {
    fn evaluate(&self, pos: &Position) -> i32;
}

/// Convert a White-centric evaluator score into side-to-move perspective.
///
/// Negamax expects every node score to be from the perspective of the player
/// to move in `pos`.
#[inline]
pub fn evaluate_for_side_to_move<E: Evaluator>(evaluator: &E, pos: &Position) -> i32 {
    let white_centric = evaluator.evaluate(pos);
    if pos.side_to_move == Color::White {
        white_centric
    } else {
        -white_centric
    }
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

        score += evaluate_bishop_pair(pos);
        score += evaluate_pawn_structure(pos);

        score
    }
}

fn evaluate_bishop_pair(pos: &Position) -> i32 {
    let white_bishops = pos
        .pieces
        .get(Piece::from_parts(Color::White, Some(PieceKind::Bishop)))
        .count_ones() as i32;
    let black_bishops = pos
        .pieces
        .get(Piece::from_parts(Color::Black, Some(PieceKind::Bishop)))
        .count_ones() as i32;

    let white_bonus = if white_bishops >= 2 {
        BISHOP_PAIR_BONUS
    } else {
        0
    };
    let black_bonus = if black_bishops >= 2 {
        BISHOP_PAIR_BONUS
    } else {
        0
    };

    white_bonus - black_bonus
}

fn evaluate_pawn_structure(pos: &Position) -> i32 {
    evaluate_pawn_structure_for_color(pos, Color::White)
        - evaluate_pawn_structure_for_color(pos, Color::Black)
}

fn evaluate_pawn_structure_for_color(pos: &Position, color: Color) -> i32 {
    let our_pawns = pos
        .pieces
        .get(Piece::from_parts(color, Some(PieceKind::Pawn)));
    let their_pawns = pos
        .pieces
        .get(Piece::from_parts(color.opposite(), Some(PieceKind::Pawn)));

    if our_pawns.is_empty() {
        return 0;
    }

    let mut score = 0;
    let mut file_counts = [0u8; 8];

    for sq in our_pawns.squares() {
        file_counts[sq.file() as usize] = file_counts[sq.file() as usize].saturating_add(1);
    }

    for sq in our_pawns.squares() {
        let file = sq.file() as usize;
        let rank = sq.rank() as usize;

        if file_counts[file] > 1 {
            score -= DOUBLED_PAWN_PENALTY;
        }

        let left_file_has_pawn = file > 0 && file_counts[file - 1] > 0;
        let right_file_has_pawn = file < 7 && file_counts[file + 1] > 0;
        if !left_file_has_pawn && !right_file_has_pawn {
            score -= ISOLATED_PAWN_PENALTY;
        }

        if is_passed_pawn(sq, color, their_pawns.0) {
            let advance = if color == Color::White {
                rank
            } else {
                7usize.saturating_sub(rank)
            };
            score += PASSED_PAWN_BONUS_BY_ADVANCE[advance.min(7)];
        }
    }

    score
}

fn is_passed_pawn(sq: bitboard::Square, color: Color, enemy_pawns_bits: u64) -> bool {
    let file = sq.file() as i32;
    let rank = sq.rank() as i32;

    let file_start = (file - 1).max(0);
    let file_end = (file + 1).min(7);

    for ef in file_start..=file_end {
        for er in 0..8 {
            let ahead = match color {
                Color::White => er > rank,
                Color::Black => er < rank,
            };
            if !ahead {
                continue;
            }

            let idx = (er * 8 + ef) as u64;
            if (enemy_pawns_bits & (1u64 << idx)) != 0 {
                return false;
            }
        }
    }

    true
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
    // phase = MAX_PHASE when full material (midgame)
    // phase = 0 when minimal material (endgame)
    // So weight MID when phase is high, END when phase is low
    ((mid * (MAX_PHASE - phase)) + (end * phase)) / MAX_PHASE
}
