use crate::core::arena::Arena;
use crate::search::evaluator::Evaluator;
use crate::search::evaluator::evaluate_for_side_to_move;
use crate::search::see::compute_see;
use bitboard::MoveList;
use bitboard::mov::ChessMove;
use bitboard::movegen::MoveGenerator;
use bitboard::movegen::generate_legal_moves_fast;
use bitboard::movegen::generate_pseudo_captures_fast;
use bitboard::piece::Color;
use bitboard::piece::Piece;
use bitboard::piece::PieceKind;
use bitboard::position::Position;

const MAX_QSEARCH_DEPTH: usize = 8;
const MAX_QSEARCH_DEPTH_DENSE: usize = 6;
const HIGH_DENSITY_PIECE_COUNT: u32 = 24;
const DELTA_MARGIN: i32 = 200; // Queen value margin for delta pruning
const CHECK_GEN_DEPTH_LIMIT: Option<usize> = None; // None => disable non-capture check generation in qsearch
/// SEE threshold: prune captures with SEE worse than this value (in centipawns)
/// -100 means allow trades of equal value; more negative allows bigger losses
const SEE_QUIET_THRESHOLD: i32 = -50;
/// SEE threshold in deeper qsearch - tighter pruning to avoid explosion
const SEE_DEEP_THRESHOLD: i32 = 0;
/// In very crowded boards, require captures to have clear tactical value.
const SEE_DENSE_THRESHOLD: i32 = 100;

const fn should_run_full_see(attacker_value: i32, victim_value: i32, threshold: i32) -> bool {
    // If material swing already clears threshold with attacker/victim values,
    // skip full SEE recursion and keep the move.
    victim_value - attacker_value < threshold
}

pub fn quiescence_with_arena<M: MoveGenerator, E: Evaluator>(
    movegen: &M,
    evaluator: &E,
    arena: &mut Arena,
    ply: usize,
    alpha: i32,
    beta: i32,
) -> i32 {
    quiescence_internal(movegen, evaluator, arena, ply, alpha, beta, 0)
}

fn quiescence_internal<M: MoveGenerator, E: Evaluator>(
    movegen: &M,
    evaluator: &E,
    arena: &mut Arena,
    ply: usize,
    mut alpha: i32,
    beta: i32,
    qsearch_depth: usize,
) -> i32 {
    crate::search::core::update_seldepth(ply);

    let pos = arena.get(ply).position;
    let is_dense_position = pos.all_pieces().count() >= HIGH_DENSITY_PIECE_COUNT;
    let qsearch_depth_cap = if is_dense_position {
        MAX_QSEARCH_DEPTH_DENSE
    } else {
        MAX_QSEARCH_DEPTH
    };

    // Prevent infinite qsearch recursion
    if qsearch_depth >= qsearch_depth_cap {
        return evaluate_for_side_to_move(evaluator, &pos);
    }
    let in_check = movegen.in_check(&pos);

    // When in check, we MUST search (can't stand pat), otherwise stand pat is valid
    let stand_pat = if !in_check {
        evaluate_for_side_to_move(evaluator, &pos)
    } else {
        i32::MIN + 1 // Placeholder, must search when in check
    };

    if !in_check && stand_pat >= beta {
        return stand_pat;
    }

    // Update alpha with stand-pat score if not in check (branchless)
    if !in_check {
        alpha = alpha.max(stand_pat);
    }

    // If we're in check, we must consider all evasions (not just captures)
    // Otherwise, search captures and (at shallow depth) checking moves
    // Use MoveList for stack allocation (avoids heap in hot path)
    let mut moves = if in_check {
        generate_legal_moves_fast(&pos)
    } else {
        // Generate captures first
        let move_list = generate_pseudo_captures_fast(&pos);

        // Filter with delta and SEE pruning
        let mut filtered = MoveList::new();
        for i in 0..move_list.len() {
            let m = move_list[i];
            // Delta pruning: skip captures that can't possibly improve alpha
            // even if we capture the target piece
            let victim = get_piece_on_square(&pos, m.to);
            if victim != Piece::None {
                let victim_val = piece_value(victim.kind());
                // If stand_pat + captured piece value + margin is still below alpha, prune
                if stand_pat + victim_val + DELTA_MARGIN < alpha {
                    continue;
                }

                // SEE pruning: skip bad captures on high-density boards
                // Adjust threshold based on qsearch depth - tighter at depth > 2
                let see_threshold = if is_dense_position {
                    SEE_DENSE_THRESHOLD
                } else if qsearch_depth >= 2 {
                    SEE_DEEP_THRESHOLD
                } else {
                    SEE_QUIET_THRESHOLD
                };

                // Run full SEE only for likely-losing captures; this
                // avoids expensive recursive SEE on clearly favorable trades.
                let attacker = get_piece_on_square(&pos, m.from);
                if attacker != Piece::None {
                    let attacker_val = piece_value(attacker.kind());

                    // In high-density tactical skirmishes, avoid neutral
                    // or losing exchanges below the first qsearch layer.
                    if is_dense_position && qsearch_depth >= 1 && victim_val <= attacker_val {
                        continue;
                    }

                    if should_run_full_see(attacker_val, victim_val, see_threshold) {
                        let see_value = compute_see(&pos, m.from, m.to);
                        if see_value < see_threshold {
                            continue;
                        }
                    }
                }
            }

            filtered.push(m);
        }

        // At shallow qsearch depth, also generate checking moves
        // to catch tactical shots that would otherwise be over the horizon
        if let Some(limit) = CHECK_GEN_DEPTH_LIMIT
            && qsearch_depth < limit
        {
            let all_quiet = generate_legal_moves_fast(&pos);
            for i in 0..all_quiet.len() {
                let m = all_quiet[i];
                // Skip if already in move list (captures/promotions)
                let mut already_has = false;
                for j in 0..filtered.len() {
                    if filtered[j] == m {
                        already_has = true;
                        break;
                    }
                }
                if already_has {
                    continue;
                }

                // Check if this move gives check
                let mut temp = pos;
                pos.apply_move_into(&m, &mut temp);
                if movegen.in_check(&temp) {
                    filtered.push(m);
                }
            }
        }

        filtered
    };

    if moves.is_empty() {
        if in_check {
            // Checkmate: return a mate score adjusted by ply
            return -30_000 + ply as i32;
        }
        return stand_pat;
    }

    // Cache MVV/LVA scores once per move to avoid repeated piece lookups during
    // sorting
    let num_moves = moves.len();
    let mut scores: [i32; 256] = [0; 256];
    for i in 0..num_moves {
        scores[i] = mvv_lva_score(&pos, &moves[i]);
    }

    // Insertion sort by descending MVV/LVA score (higher victim, lower attacker is
    // better) This is cache-friendly for small move counts typical in qsearch
    // (< 10 captures)
    let moves_slice = moves.as_mut_slice();
    for i in 1..num_moves {
        let key_score = scores[i];
        let key_move = moves_slice[i];
        let mut j = i;
        while j > 0 && scores[j - 1] < key_score {
            moves_slice[j] = moves_slice[j - 1];
            scores[j] = scores[j - 1];
            j -= 1;
        }
        moves_slice[j] = key_move;
        scores[j] = key_score;
    }

    let mut best = i32::MIN;
    for i in 0..moves.len() {
        let m = moves[i];
        {
            let (parent, child) = arena.get_pair_mut(ply, ply + 1);
            parent.position.apply_move_into(&m, &mut child.position);

            // In the non-check qsearch branch we start from pseudo-captures,
            // so verify legality after the real move apply to avoid duplicate work.
            if !in_check {
                let mut legal_test = child.position;
                legal_test.side_to_move = parent.position.side_to_move;
                if movegen.in_check(&legal_test) {
                    continue;
                }
            }
        }

        let score = -quiescence_internal(
            movegen,
            evaluator,
            arena,
            ply + 1,
            -beta,
            -alpha,
            qsearch_depth + 1,
        );

        best = best.max(score);
        alpha = alpha.max(score);

        if alpha >= beta {
            break;
        }
    }

    if best == i32::MIN { stand_pat } else { best }
}

// MVV/LVA score: higher is better.
fn mvv_lva_score(pos: &Position, mv: &ChessMove) -> i32 {
    let victim_piece = match mv.move_type {
        bitboard::mov::MoveType::EnPassant => {
            let us = pos.side_to_move;
            let cap_sq = match us {
                Color::White => mv.to.backward(1).unwrap(),
                Color::Black => mv.to.forward(1).unwrap(),
            };
            get_piece_on_square(pos, cap_sq)
        }
        _ => get_piece_on_square(pos, mv.to),
    };

    let victim_value = if victim_piece != Piece::None {
        piece_value(victim_piece.kind())
    } else {
        0
    };
    let attacker_piece = get_piece_on_square(pos, mv.from);
    let attacker_value = if attacker_piece != Piece::None {
        piece_value(attacker_piece.kind())
    } else {
        0
    };

    victim_value * 100 - attacker_value
}

const fn piece_value(kind: PieceKind) -> i32 {
    match kind {
        PieceKind::Pawn => 100,
        PieceKind::Knight => 320,
        PieceKind::Bishop => 330,
        PieceKind::Rook => 500,
        PieceKind::Queen => 900,
        PieceKind::King => 10000,
    }
}

fn get_piece_on_square(pos: &Position, sq: bitboard::Square) -> Piece {
    pos.piece_at_square(sq)
}
