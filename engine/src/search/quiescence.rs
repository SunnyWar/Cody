use crate::VERBOSE;
use crate::core::arena::Arena;
use crate::search::evaluator::Evaluator;
use crate::search::evaluator::evaluate_for_side_to_move;
use crate::util;
use bitboard::mov::ChessMove;
use bitboard::movegen::MoveGenerator;
use bitboard::piece::Color;
use bitboard::piece::Piece;
use bitboard::piece::PieceKind;
use bitboard::position::Position;
use smallvec::SmallVec;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::atomic::Ordering;

const MAX_QSEARCH_DEPTH: usize = 16;
const DELTA_MARGIN: i32 = 200; // Queen value margin for delta pruning
const MAX_CHECK_DEPTH: usize = 1; // Only generate checks at shallow qsearch depth

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

    // Prevent infinite qsearch recursion
    if qsearch_depth >= MAX_QSEARCH_DEPTH {
        return evaluate_for_side_to_move(evaluator, &arena.get(ply).position);
    }

    // Stand pat evaluation
    if VERBOSE.load(Ordering::Relaxed) {
        eprintln!("[debug] quiescence enter ply={}", ply);
        // Append to cody_uci.log for traceability
        if let Ok(mut f) = OpenOptions::new()
            .create(true)
            .append(true)
            .open("cody_uci.log")
        {
            let stamp = util::iso_stamp_ms();
            let _ = writeln!(f, "{} OUT: [debug] quiescence enter ply={}", stamp, ply);
        }
    }

    let pos = arena.get(ply).position;
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
    if !in_check && alpha < stand_pat {
        alpha = stand_pat;
    }

    // If we're in check, we must consider all evasions (not just captures)
    // Otherwise, search captures and (at shallow depth) checking moves
    // Use SmallVec to keep typical capture counts (<32) on the stack, avoiding heap
    // allocation.
    let mut moves: SmallVec<[ChessMove; 64]> = if in_check {
        bitboard::movegen::generate_legal_moves(&pos)
            .into_iter()
            .collect()
    } else {
        // Generate captures first
        let mut move_list: SmallVec<[ChessMove; 64]> =
            bitboard::movegen::generate_pseudo_captures(&pos)
                .into_iter()
                .filter(|m| {
                    // Delta pruning: skip captures that can't possibly improve alpha
                    // even if we capture the target piece
                    if let Some(victim) = get_piece_on_square(&pos, m.to) {
                        let victim_val = piece_value(victim.kind());
                        // If stand_pat + captured piece value + margin is still below alpha, prune
                        if stand_pat + victim_val + DELTA_MARGIN < alpha {
                            return false;
                        }
                    }

                    let mut temp = pos;
                    pos.apply_move_into(m, &mut temp);
                    !movegen.in_check(&temp)
                })
                .collect();

        // At shallow qsearch depth, also generate checking moves
        // to catch tactical shots that would otherwise be over the horizon
        if qsearch_depth < MAX_CHECK_DEPTH {
            let all_quiet = bitboard::movegen::generate_legal_moves(&pos);
            for m in all_quiet {
                // Skip if already in move list (captures/promotions)
                if move_list.contains(&m) {
                    continue;
                }

                // Check if this move gives check
                let mut temp = pos;
                pos.apply_move_into(&m, &mut temp);
                if movegen.in_check(&temp) {
                    move_list.push(m);
                }
            }
        }

        move_list
    };

    if moves.is_empty() {
        if in_check {
            // Checkmate: return a mate score adjusted by ply
            return -30_000 + ply as i32;
        }
        return stand_pat;
    }

    // Order captures by MVV/LVA descending
    moves.sort_by_key(|m| -mvv_lva_score(&pos, m));

    let mut best = i32::MIN;
    for m in moves {
        {
            let (parent, child) = arena.get_pair_mut(ply, ply + 1);
            parent.position.apply_move_into(&m, &mut child.position);
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

        if score > best {
            best = score;
        }

        if score > alpha {
            alpha = score;
        }

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

    let victim_value = victim_piece.map(|p| piece_value(p.kind())).unwrap_or(0);
    let attacker_piece = get_piece_on_square(pos, mv.from);
    let attacker_value = attacker_piece.map(|p| piece_value(p.kind())).unwrap_or(0);

    victim_value * 100 - attacker_value
}

fn piece_value(kind: PieceKind) -> i32 {
    match kind {
        PieceKind::Pawn => 100,
        PieceKind::Knight => 320,
        PieceKind::Bishop => 330,
        PieceKind::Rook => 500,
        PieceKind::Queen => 900,
        PieceKind::King => 10000,
    }
}

fn get_piece_on_square(pos: &Position, sq: bitboard::Square) -> Option<Piece> {
    let mask = bitboard::BitBoardMask::from_square(sq);
    for (piece, bb) in pos.pieces.iter() {
        if (bb & mask).is_nonempty() {
            return Some(piece);
        }
    }
    None
}
