// src/search/engine.rs

use crate::core::arena::Arena;
use crate::search::evaluator::Evaluator;
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
                        let score =
                            -search_node_with_arena(&mg, &ev, &mut local_arena, 1, depth - 1);
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
) -> i32 {
    NODE_COUNT.fetch_add(1, Ordering::Relaxed);

    if remaining == 0 {
        return evaluator.evaluate(&arena.get(ply).position);
    }

    let moves = {
        let (parent, _) = arena.get_pair_mut(ply, ply + 1);
        generate_legal_moves(&parent.position)
    };

    if moves.is_empty() {
        let pos = &arena.get(ply).position;
        if movegen.in_check(pos) {
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
        let score = -search_node_with_arena(movegen, evaluator, arena, ply + 1, remaining - 1);
        best_score = best_score.max(score);
    }

    best_score
}
