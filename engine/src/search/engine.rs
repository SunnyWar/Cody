// src/search/engine.rs

use crate::core::arena::Arena;
use crate::search::evaluator::Evaluator;
use bitboard::piece::{Color, Piece, PieceKind};
use bitboard::{
    mov::ChessMove,
    movegen::{MoveGenerator, generate_legal_moves},
    position::Position,
};
use std::io::{self, Write};
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

pub static NODE_COUNT: AtomicU64 = AtomicU64::new(0);

// Positive large value used to detect mate scores. Keep consistent with UCI API's MATE_SCORE.
const MATE_SCORE: i32 = 30_000;
// Large infinity value for alpha-beta bounds
const INF: i32 = 1_000_000_000;

pub struct Engine<
    M: MoveGenerator + Clone + Send + Sync + 'static,
    E: Evaluator + Clone + Send + Sync + 'static,
> {
    arena: Arena,
    movegen: M,
    evaluator: E,
    arena_capacity: usize,
    num_threads: usize,
}

impl<M: MoveGenerator + Clone + Send + Sync + 'static, E: Evaluator + Clone + Send + Sync + 'static>
    Engine<M, E>
{
    pub fn new(arena_size: usize, movegen: M, evaluator: E) -> Self {
        Self {
            arena: Arena::new(arena_size),
            movegen,
            evaluator,
            arena_capacity: arena_size,
            num_threads: 1,
        }
    }

    /// Set number of threads to use for root parallelism. 1 = serial.
    pub fn set_num_threads(&mut self, n: usize) {
        self.num_threads = n.max(1);
    }

    pub fn search(&mut self, root: &Position, depth: usize) -> (ChessMove, i32) {
        self.arena.reset();
        self.arena.get_mut(0).position.copy_from(root);

        let moves = {
            let (parent, _) = self.arena.get_pair_mut(0, 1);
            generate_legal_moves(&parent.position)
        };

        if moves.is_empty() {
            let score = if self.movegen.in_check(root) {
                -MATE_SCORE
            } else {
                0
            };
            return (ChessMove::null(), score);
        }

        let mut best_score = i32::MIN;
        let mut best_move = ChessMove::null();

        if self.num_threads <= 1 {
            // Serial path
            for m in moves {
                {
                    let (parent, child) = self.arena.get_pair_mut(0, 1);
                    parent.position.apply_move_into(&m, &mut child.position);
                }
                let score = -search_node_with_arena(
                    &self.movegen,
                    &self.evaluator,
                    &mut self.arena,
                    1,
                    depth - 1,
                    -INF,
                    INF,
                );

                if score > best_score {
                    best_score = score;
                    best_move = m;

                    // Build PV string — for now just the root move
                    let pv_str = best_move.to_string();
                    print_uci_info(depth, best_score, &pv_str, 0);
                }
            }
        } else {
            // Parallel root move evaluation using rayon
            use rayon::prelude::*;

            // Make a thread pool with the requested number of threads
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(self.num_threads)
                .build()
                .expect("Failed to build rayon thread pool");

            // Clone components into the closure so each thread owns its data.
            let mg = self.movegen.clone();
            let ev = self.evaluator.clone();
            let arena_cap = self.arena_capacity;

            let results: Vec<(ChessMove, i32)> = pool.install(|| {
                moves
                    .into_par_iter()
                    .map(move |m| {
                        // Each thread gets its own arena to avoid synchronization
                        let mut local_arena = Arena::new(arena_cap);
                        local_arena.get_mut(0).position.copy_from(root);
                        {
                            let (parent, child) = local_arena.get_pair_mut(0, 1);
                            parent.position.apply_move_into(&m, &mut child.position);
                        }
                        let score = -search_node_with_arena(
                            &mg,
                            &ev,
                            &mut local_arena,
                            1,
                            depth - 1,
                            -INF,
                            INF,
                        );
                        (m, score)
                    })
                    .collect()
            });

            for (m, score) in results {
                if score > best_score {
                    best_score = score;
                    best_move = m;
                    let pv_str = best_move.to_string();
                    print_uci_info(depth, best_score, &pv_str, 0);
                }
            }
        }

        (best_move, best_score)
    }

    fn search_node(&mut self, ply: usize, remaining: usize) -> i32 {
        // Default wrapper that uses the internal arena and components
        search_node_with_arena(
            &self.movegen,
            &self.evaluator,
            &mut self.arena,
            ply,
            remaining,
            -INF,
            INF,
        )
    }

    pub fn clear_state(&self) {
        NODE_COUNT.store(0, Ordering::Relaxed)
    }
}

pub fn print_uci_info(
    depth: usize,
    score: i32,
    pv: &str, // principal variation as a space-separated string
    elapsed_ms: u64,
) {
    let nodes = NODE_COUNT.load(Ordering::Relaxed);
    let nps = if elapsed_ms > 0 {
        nodes * 1000 / elapsed_ms
    } else {
        0
    };

    if score.abs() > MATE_SCORE - 100 {
        let mate_in = if score > 0 {
            (MATE_SCORE - score + 1) / 2
        } else {
            -(MATE_SCORE + score) / 2
        };
        println!(
            "info depth {} score mate {} nodes {} time {} nps {} pv {}",
            depth, mate_in, nodes, elapsed_ms, nps, pv
        );
    } else {
        println!(
            "info depth {} score cp {} nodes {} time {} nps {} pv {}",
            depth, score, nodes, elapsed_ms, nps, pv
        );
    }

    io::stdout().flush().unwrap();
}

// Helper recursive search that operates on a provided arena and components.
fn search_node_with_arena<M: MoveGenerator, E: Evaluator>(
    movegen: &M,
    evaluator: &E,
    arena: &mut Arena,
    ply: usize,
    remaining: usize,
    mut alpha: i32,
    beta: i32,
) -> i32 {
    NODE_COUNT.fetch_add(1, Ordering::Relaxed);

    if remaining == 0 {
        return quiescence_with_arena(movegen, evaluator, arena, ply, alpha, beta);
    }

    let moves = {
        let (parent, _) = arena.get_pair_mut(ply, ply + 1);
        generate_legal_moves(&parent.position)
    };

    if moves.is_empty() {
        let pos = &arena.get(ply).position;
        if movegen.in_check(pos) {
            // mate: return losing score adjusted by ply (so earlier mate is worse)
            return -MATE_SCORE + ply as i32;
        }
        return 0;
    }

    let mut best_score = i32::MIN;
    for m in moves {
        {
            let (parent, child) = arena.get_pair_mut(ply, ply + 1);
            parent.position.apply_move_into(&m, &mut child.position);
        }

        // Recursive negamax with swapped alpha/beta and sign inversion
        let score = -search_node_with_arena(
            movegen,
            evaluator,
            arena,
            ply + 1,
            remaining - 1,
            -beta,
            -alpha,
        );

        if score > best_score {
            best_score = score;
        }

        if score > alpha {
            alpha = score;
        }

        // Beta cutoff
        if alpha >= beta {
            break;
        }
    }

    best_score
}

// Quiescence search that explores captures (and promotions / en-passant). Also handles being in check by
// generating all legal moves when in check so evasions are searched.
fn quiescence_with_arena<M: MoveGenerator, E: Evaluator>(
    movegen: &M,
    evaluator: &E,
    arena: &mut Arena,
    ply: usize,
    mut alpha: i32,
    beta: i32,
) -> i32 {
    // Stand pat evaluation
    let stand_pat = evaluator.evaluate(&arena.get(ply).position);
    if stand_pat >= beta {
        return stand_pat;
    }
    if alpha < stand_pat {
        alpha = stand_pat;
    }

    let (parent, _) = arena.get_pair_mut(ply, ply + 1);
    let pos = &parent.position;

    // If we're in check, we must consider all evasions (not just captures)
    let mut moves = if movegen.in_check(pos) {
        generate_legal_moves(pos)
    } else {
        // Otherwise, generate captures/promotions/en-passant only
        generate_legal_moves(pos)
            .into_iter()
            .filter(|m| is_capture_like_move(m))
            .collect()
    };

    if moves.is_empty() {
        return stand_pat;
    }

    // Order captures by MVV/LVA descending
    moves.sort_by_key(|m| -mvv_lva_score(pos, m));

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

// Determine whether a move should be considered in quiescence (captures, en-passant, promotions)
fn is_capture_like_move(mv: &ChessMove) -> bool {
    use bitboard::mov::MoveType;

    match mv.move_type {
        MoveType::Capture | MoveType::EnPassant => true,
        MoveType::Promotion(_) => true,
        MoveType::Quiet | MoveType::CastleKingside | MoveType::CastleQueenside => false,
    }
}

// MVV/LVA score: higher is better. Use victim material scaled minus attacker material.
fn mvv_lva_score(pos: &Position, mv: &ChessMove) -> i32 {
    // victim
    let victim_piece = match mv.move_type {
        bitboard::mov::MoveType::EnPassant => {
            // captured pawn is behind the to-square depending on side to move
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

    // attacker
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

// Find the piece enum occupying a square in the given position, if any.
fn get_piece_on_square(pos: &Position, sq: bitboard::Square) -> Option<Piece> {
    let mask = bitboard::BitBoardMask::from_square(sq);
    for (piece, bb) in pos.pieces.iter() {
        if (bb & mask).is_nonempty() {
            return Some(piece);
        }
    }
    None
}
