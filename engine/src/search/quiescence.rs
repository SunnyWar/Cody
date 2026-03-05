use crate::VERBOSE;
use crate::core::arena::Arena;
use crate::search::evaluator::Evaluator;
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

pub fn quiescence_with_arena<M: MoveGenerator, E: Evaluator>(
    movegen: &M,
    evaluator: &E,
    arena: &mut Arena,
    ply: usize,
    mut alpha: i32,
    beta: i32,
) -> i32 {
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
    let stand_pat = evaluator.evaluate(&arena.get(ply).position);
    if stand_pat >= beta {
        return stand_pat;
    }
    if alpha < stand_pat {
        alpha = stand_pat;
    }

    let pos = arena.get(ply).position;

    let in_check = movegen.in_check(&pos);

    // If we're in check, we must consider all evasions (not just captures)
    // Use SmallVec to keep typical capture counts (<32) on the stack, avoiding heap
    // allocation.
    let mut moves: SmallVec<[ChessMove; 32]> = if in_check {
        bitboard::movegen::generate_legal_moves(&pos)
            .into_iter()
            .collect()
    } else {
        bitboard::movegen::generate_pseudo_captures(&pos)
            .into_iter()
            .filter(|m| {
                // `apply_move_into` writes all state fields, so cloning the
                // current position avoids expensive default FEN parsing.
                let mut temp = pos;
                pos.apply_move_into(m, &mut temp);
                !movegen.in_check(&temp)
            })
            .collect()
    };

    if moves.is_empty() {
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

        let score = -quiescence_with_arena(movegen, evaluator, arena, ply + 1, -beta, -alpha);

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
