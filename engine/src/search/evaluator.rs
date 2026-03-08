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
pub fn evaluate_for_side_to_move<E: Evaluator>(evaluator: &E, pos: &Position) -> i32 {
    let white_centric = evaluator.evaluate(pos);
    // Branchless conditional negation:
    // - White to move: return x
    // - Black to move: return -x (wrapping-safe)
    let flip = (pos.side_to_move == Color::Black) as i32;
    (white_centric ^ -flip).wrapping_add(flip)
}

impl Evaluator for MaterialEvaluator {
    fn evaluate(&self, pos: &Position) -> i32 {
        let mut score = 0;
        let phase = compute_phase(pos);

        for &color in &[Color::White, Color::Black] {
            let sign = if color == Color::White { 1 } else { -1 };

            // Process each piece type
            // Pawns often come in batches of 8, perfect for SIMD
            {
                let piece = Piece::from_parts(color, Some(PieceKind::Pawn));
                let bb = pos.pieces.get(piece);
                // Pawns are bounded (max 8 per side), so stack storage avoids
                // per-evaluation heap allocation in this hot path.
                let mut indices = [0usize; 16];
                let mut count = 0usize;

                for sq in bb.squares() {
                    let idx = if color == Color::White {
                        sq.index()
                    } else {
                        63 - sq.index()
                    };
                    indices[count] = idx;
                    count += 1;
                }

                if count != 0 {
                    score += evaluate_pieces_batch_simd(
                        &indices[..count],
                        &PAWN_SQUARE_TABLE,
                        &PAWN_ENDGAME_TABLE,
                        PIECE_VALUES[PieceKind::Pawn as usize],
                        phase,
                        sign,
                    );
                }
            }

            // Process other pieces (typically fewer, but SIMD still helps)
            for kind in [
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
                        _ => 0,
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
    // SIMD-optimized: count bishops for both sides in parallel
    let white_bishops_bb = pos
        .pieces
        .get(Piece::from_parts(Color::White, Some(PieceKind::Bishop)))
        .0;
    let black_bishops_bb = pos
        .pieces
        .get(Piece::from_parts(Color::Black, Some(PieceKind::Bishop)))
        .0;

    // Use SIMD for parallel popcount (pads with zeros for unused lanes)
    let vec = bitboard::intrinsics::SimdU64x4::new(white_bishops_bb, black_bishops_bb, 0, 0);
    let counts = vec.popcnt_parallel();

    let white_bishops = counts[0] as i32;
    let black_bishops = counts[1] as i32;

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

    // SIMD-optimized phase computation: count all pieces for both sides in parallel
    for color in [Color::White, Color::Black] {
        // Load 4 piece bitboards into SIMD vector for parallel popcount
        let pawns = pos.pieces.get(Piece::from_parts(color, Some(Pawn))).0;
        let knights = pos.pieces.get(Piece::from_parts(color, Some(Knight))).0;
        let bishops = pos.pieces.get(Piece::from_parts(color, Some(Bishop))).0;
        let rooks = pos.pieces.get(Piece::from_parts(color, Some(Rook))).0;

        // Parallel popcount on 4 bitboards simultaneously
        let vec = bitboard::intrinsics::SimdU64x4::new(pawns, knights, bishops, rooks);
        let counts = vec.popcnt_parallel();

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

        // Use SIMD results
        let pawn_count = counts[0] as i32;
        let knight_count = counts[1] as i32;
        let bishop_count = counts[2] as i32;
        let rook_count = counts[3] as i32;

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

/// SIMD-optimized batch PST evaluation.
///
/// Process up to 8 pieces at once using AVX2 SIMD operations.
/// Returns the sum of all piece values + PST bonuses.
fn evaluate_pieces_batch_simd(
    indices: &[usize],
    mid_table: &[i32; 64],
    end_table: &[i32; 64],
    piece_value: i32,
    phase: i32,
    sign: i32,
) -> i32 {
    let count = indices.len();
    if count == 0 {
        return 0;
    }

    // For 8 or more pieces, use SIMD batch processing
    if count >= 8 {
        let mut batch_score = 0;
        let mut i = 0;

        while i + 8 <= count {
            // Load 8 midgame PST values
            let mid_vec = bitboard::intrinsics::SimdI32x8::new([
                mid_table[indices[i]],
                mid_table[indices[i + 1]],
                mid_table[indices[i + 2]],
                mid_table[indices[i + 3]],
                mid_table[indices[i + 4]],
                mid_table[indices[i + 5]],
                mid_table[indices[i + 6]],
                mid_table[indices[i + 7]],
            ]);

            // Load 8 endgame PST values
            let end_vec = bitboard::intrinsics::SimdI32x8::new([
                end_table[indices[i]],
                end_table[indices[i + 1]],
                end_table[indices[i + 2]],
                end_table[indices[i + 3]],
                end_table[indices[i + 4]],
                end_table[indices[i + 5]],
                end_table[indices[i + 6]],
                end_table[indices[i + 7]],
            ]);

            // Blend: ((mid * (MAX_PHASE - phase)) + (end * phase)) / MAX_PHASE
            let max_phase_vec = bitboard::intrinsics::SimdI32x8::splat(MAX_PHASE);
            let phase_vec = bitboard::intrinsics::SimdI32x8::splat(phase);
            let inv_phase_vec = max_phase_vec - phase_vec;

            // mid * (MAX_PHASE - phase)
            let mid_weighted = bitboard::intrinsics::SimdI32x8::new([
                mid_vec.data[0] * inv_phase_vec.data[0],
                mid_vec.data[1] * inv_phase_vec.data[1],
                mid_vec.data[2] * inv_phase_vec.data[2],
                mid_vec.data[3] * inv_phase_vec.data[3],
                mid_vec.data[4] * inv_phase_vec.data[4],
                mid_vec.data[5] * inv_phase_vec.data[5],
                mid_vec.data[6] * inv_phase_vec.data[6],
                mid_vec.data[7] * inv_phase_vec.data[7],
            ]);

            // end * phase
            let end_weighted = bitboard::intrinsics::SimdI32x8::new([
                end_vec.data[0] * phase_vec.data[0],
                end_vec.data[1] * phase_vec.data[1],
                end_vec.data[2] * phase_vec.data[2],
                end_vec.data[3] * phase_vec.data[3],
                end_vec.data[4] * phase_vec.data[4],
                end_vec.data[5] * phase_vec.data[5],
                end_vec.data[6] * phase_vec.data[6],
                end_vec.data[7] * phase_vec.data[7],
            ]);

            // Add and divide
            let blended = mid_weighted + end_weighted;
            let pst_bonuses = bitboard::intrinsics::SimdI32x8::new([
                blended.data[0] / MAX_PHASE,
                blended.data[1] / MAX_PHASE,
                blended.data[2] / MAX_PHASE,
                blended.data[3] / MAX_PHASE,
                blended.data[4] / MAX_PHASE,
                blended.data[5] / MAX_PHASE,
                blended.data[6] / MAX_PHASE,
                blended.data[7] / MAX_PHASE,
            ]);

            // Add piece values
            let piece_values = bitboard::intrinsics::SimdI32x8::splat(piece_value);
            let total_values = pst_bonuses + piece_values;

            // Sum horizontally and apply sign
            batch_score += sign * total_values.horizontal_sum();
            i += 8;
        }

        // Handle remainder
        for idx in &indices[i..] {
            let pst_bonus = blend_tables(mid_table[*idx], end_table[*idx], phase);
            batch_score += sign * (piece_value + pst_bonus);
        }

        return batch_score;
    }

    // Fallback for < 8 pieces: use scalar evaluation
    let mut score = 0;
    for &idx in indices {
        let pst_bonus = blend_tables(mid_table[idx], end_table[idx], phase);
        score += sign * (piece_value + pst_bonus);
    }
    score
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
            let file_mask = 0x0101_0101_0101_0101u64 << (king_file as u32);
            let our_pawn_on_file = (our_pawns.0 & file_mask) != 0;
            let enemy_pawn_on_file = (enemy_pawns.0 & file_mask) != 0;
            let semi_open_file_penalty = if !our_pawn_on_file && enemy_pawn_on_file {
                OPEN_FILE_NEAR_KING
            } else {
                0
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
            let file_mask = 0x0101_0101_0101_0101u64 << (file as u32);
            let our_pawn_on_file = (our_pawns.0 & file_mask) != 0;
            let enemy_pawn_on_file = (enemy_pawns.0 & file_mask) != 0;

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
