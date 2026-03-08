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

// Piece mobility (activity bonus per legal move a piece can make)
const MOBILITY_WEIGHT: i32 = 4;

// King safety penalties (centipawns per issue)
const EXPOSED_KING_PENALTY: i32 = 25; // King on open file/rank
const OPEN_FILE_NEAR_KING: i32 = 15; // Semi-open file near king
const KING_LACKING_ESCAPE_SQUARES: i32 = 20; // King with few escape squares
// Rook activity bonuses
const ROOK_ON_OPEN_FILE_BONUS: i32 = 20; // Rook with no pawns on file
const ROOK_ON_SEMIOPEN_FILE_BONUS: i32 = 10; // Rook with enemy pawns only

// Advanced pawn promotion bonuses (heavily weighted to encourage winning
// endgames)
const PAWN_NEAR_PROMOTION: [i32; 8] = [0, 0, 0, 8, 20, 60, 150, 0];
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
        score += evaluate_mobility(pos);
        score += evaluate_king_safety(pos);
        score += evaluate_rook_activity(pos);
        score += evaluate_pawn_advancement(pos);

        score
    }
}

fn evaluate_bishop_pair(pos: &Position) -> i32 {
    let white_bishops = bitboard::intrinsics::popcnt(
        pos.pieces
            .get(Piece::from_parts(Color::White, Some(PieceKind::Bishop)))
            .0,
    ) as i32;
    let black_bishops = bitboard::intrinsics::popcnt(
        pos.pieces
            .get(Piece::from_parts(Color::Black, Some(PieceKind::Bishop)))
            .0,
    ) as i32;

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

/// Evaluate piece mobility (number of squares each piece can move to).
/// Active pieces are worth more than passive pieces.
fn evaluate_mobility(pos: &Position) -> i32 {
    let mut white_mobility = 0i32;
    let mut black_mobility = 0i32;

    // Simple mobility: count checks per piece (expensive but necessary for
    // strength) For now, use a lightweight heuristic based on piece placement
    // rather than generating all legal moves.

    for color in [Color::White, Color::Black] {
        for kind in [
            PieceKind::Knight,
            PieceKind::Bishop,
            PieceKind::Rook,
            PieceKind::Queen,
        ] {
            let piece = Piece::from_parts(color, Some(kind));
            let bb = pos.pieces.get(piece);

            // Lightweight heuristic: pieces in center/active squares are better
            // than pieces on edges/passive squares.
            for sq in bb.squares() {
                let rank = sq.rank() as i32;
                let file = sq.file() as i32;

                // Center control bonus (pieces closer to center d-file & 4-5 ranks)
                let center_distance = ((file - 3).abs() + (rank - 3).abs()).max(1);
                let center_bonus = (5 - center_distance) / 2; // +2 to 0 depending on distance

                let mobility_bonus = center_bonus * MOBILITY_WEIGHT;

                if color == Color::White {
                    white_mobility += mobility_bonus;
                } else {
                    black_mobility += mobility_bonus;
                }
            }
        }
    }

    white_mobility - black_mobility
}

/// Evaluate king safety (penalize exposed kings).
fn evaluate_king_safety(pos: &Position) -> i32 {
    let mut white_safety = 0i32;
    let mut black_safety = 0i32;

    for color in [Color::White, Color::Black] {
        let king_piece = Piece::from_parts(color, Some(PieceKind::King));
        let king_bb = pos.pieces.get(king_piece);

        if king_bb.is_empty() {
            continue; // No king (shouldn't happen in legal position)
        }

        // Find king square (should be exactly one)
        if let Some(king_sq) = king_bb.squares().next() {
            let king_rank = king_sq.rank() as i32;
            let king_file = king_sq.file() as i32;

            // Penalty: king on open files/ranks (few pawns nearby)
            let our_pawns = pos
                .pieces
                .get(Piece::from_parts(color, Some(PieceKind::Pawn)));
            let enemy_pawns = pos
                .pieces
                .get(Piece::from_parts(color.opposite(), Some(PieceKind::Pawn)));

            let mut nearby_pawn_count = 0;
            for file_offset in -1..=1 {
                let check_file = king_file + file_offset;
                if !(0..=7).contains(&check_file) {
                    continue;
                }

                for rank_offset in -1..=1 {
                    let check_rank = king_rank + rank_offset;
                    if !(0..=7).contains(&check_rank) {
                        continue;
                    }

                    // Count our own pawns for shelter
                    let idx = ((check_rank * 8 + check_file) as u32) as u64;
                    if check_file != king_file && (our_pawns.0 & (1u64 << idx)) != 0 {
                        nearby_pawn_count += 1;
                    }
                }
            }

            let safety_penalty = if nearby_pawn_count < 2 {
                EXPOSED_KING_PENALTY
            } else {
                0
            };

            // Penalty: king on edge of board (fewer escape squares)
            let escape_penalty =
                if king_file == 0 || king_file == 7 || king_rank == 0 || king_rank == 7 {
                    KING_LACKING_ESCAPE_SQUARES / 2
                } else {
                    0
                };

            // Penalty: king on semi-open file (no friendly pawns but enemy pawns present)
            let semi_open_file_penalty = {
                let mut our_pawn_on_file = false;
                let mut enemy_pawn_on_file = false;

                for rank in 0..8 {
                    let sq_idx = (rank * 8 + king_file as usize) as u64;
                    if (our_pawns.0 & (1u64 << sq_idx)) != 0 {
                        our_pawn_on_file = true;
                    }
                    if (enemy_pawns.0 & (1u64 << sq_idx)) != 0 {
                        enemy_pawn_on_file = true;
                    }
                }

                if !our_pawn_on_file && enemy_pawn_on_file {
                    OPEN_FILE_NEAR_KING
                } else {
                    0
                }
            };

            let mut king_safety_penalty = safety_penalty + escape_penalty + semi_open_file_penalty;

            // Bonus for castling (if castling rights still exist, king is safer)
            let castling_bonus = if color == Color::White {
                let has_castling = pos.castling_rights.kingside(Color::White)
                    || pos.castling_rights.queenside(Color::White);
                if has_castling {
                    -EXPOSED_KING_PENALTY / 3
                } else {
                    0
                }
            } else {
                let has_castling = pos.castling_rights.kingside(Color::Black)
                    || pos.castling_rights.queenside(Color::Black);
                if has_castling {
                    -EXPOSED_KING_PENALTY / 3
                } else {
                    0
                }
            };

            king_safety_penalty += castling_bonus;

            if color == Color::White {
                white_safety += king_safety_penalty;
            } else {
                black_safety += king_safety_penalty;
            }
        }
    }

    white_safety - black_safety
}

/// Evaluate rook activity (rooks on open/semi-open files are valuable).
fn evaluate_rook_activity(pos: &Position) -> i32 {
    let mut white_bonus = 0i32;
    let mut black_bonus = 0i32;

    for color in [Color::White, Color::Black] {
        let rook_piece = Piece::from_parts(color, Some(PieceKind::Rook));
        let rooks = pos.pieces.get(rook_piece);

        let our_pawns = pos
            .pieces
            .get(Piece::from_parts(color, Some(PieceKind::Pawn)));
        let enemy_pawns = pos
            .pieces
            .get(Piece::from_parts(color.opposite(), Some(PieceKind::Pawn)));

        for rook_sq in rooks.squares() {
            let file = rook_sq.file() as usize;

            // Check if file is open or semi-open
            let mut our_pawn_on_file = false;
            let mut enemy_pawn_on_file = false;

            for rank in 0..8 {
                let sq_idx = (rank * 8 + file) as u64;
                if (our_pawns.0 & (1u64 << sq_idx)) != 0 {
                    our_pawn_on_file = true;
                }
                if (enemy_pawns.0 & (1u64 << sq_idx)) != 0 {
                    enemy_pawn_on_file = true;
                }
            }

            let bonus = if !our_pawn_on_file && !enemy_pawn_on_file {
                ROOK_ON_OPEN_FILE_BONUS // Fully open file
            } else if !our_pawn_on_file && enemy_pawn_on_file {
                ROOK_ON_SEMIOPEN_FILE_BONUS // Semi-open (enemy pawns only)
            } else {
                0 // Own pawns on file block the rook
            };

            if color == Color::White {
                white_bonus += bonus;
            } else {
                black_bonus += bonus;
            }
        }
    }

    white_bonus - black_bonus
}

/// Evaluate pawn advancement toward promotion.
/// Heavy bonus for pawns near promotion to encourage pushing for wins.
fn evaluate_pawn_advancement(pos: &Position) -> i32 {
    let mut white_bonus = 0i32;
    let mut black_bonus = 0i32;

    let white_pawns = pos
        .pieces
        .get(Piece::from_parts(Color::White, Some(PieceKind::Pawn)));
    for sq in white_pawns.squares() {
        let rank = sq.rank() as usize;
        // Rank 0 is irrelevant, rank 7 is promotion. Rank 5, 6 get bonuses
        if rank >= 4 {
            white_bonus += PAWN_NEAR_PROMOTION[rank];
        }
    }

    let black_pawns = pos
        .pieces
        .get(Piece::from_parts(Color::Black, Some(PieceKind::Pawn)));
    for sq in black_pawns.squares() {
        let rank = sq.rank() as usize;
        // For black, rank 0 is promotion, rank 7 is start. Rank 1-3 get bonuses
        if rank <= 3 {
            black_bonus += PAWN_NEAR_PROMOTION[7 - rank];
        }
    }

    white_bonus - black_bonus
}
