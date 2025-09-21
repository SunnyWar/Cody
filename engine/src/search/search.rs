// src/search/search.rs

use crate::core::arena::Arena;
use crate::search::core::{INF, MATE_SCORE, NODE_COUNT, print_uci_info, search_node_with_arena};
use crate::search::evaluator::Evaluator;
use bitboard::{
    mov::ChessMove,
    movegen::{MoveGenerator, generate_legal_moves},
    position::Position,
};
use std::sync::atomic::Ordering;

pub struct Engine<
    M: MoveGenerator + Clone + Send + Sync + 'static,
    E: Evaluator + Clone + Send + Sync + 'static,
> {
    arena: Arena,
    movegen: M,
    evaluator: E,
    arena_capacity: usize,
    num_threads: usize,
    tt: Option<crate::core::tt::TranspositionTable>,
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
            tt: Some(crate::core::tt::TranspositionTable::new(20)),
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
            // Serial path: prepare a TT reference that points at our engine TT if present,
            // otherwise to a temporary local table used only for this search.
            let mut local_tt = crate::core::tt::TranspositionTable::new(1);
            let tt_ref: &mut crate::core::tt::TranspositionTable = match self.tt.as_mut() {
                Some(ref_mut) => ref_mut,
                None => &mut local_tt,
            };

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
                    tt_ref,
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
                        // If this Engine had a TT, we don't share it with threads for now.
                        // Threads run without a TT to avoid synchronization.
                        {
                            let (parent, child) = local_arena.get_pair_mut(0, 1);
                            parent.position.apply_move_into(&m, &mut child.position);
                        }
                        // Each thread gets its own TT instance (no shared access)
                        let mut local_tt_thread = crate::core::tt::TranspositionTable::new(1);
                        let score = -search_node_with_arena(
                            &mg,
                            &ev,
                            &mut local_arena,
                            1,
                            depth - 1,
                            -INF,
                            INF,
                            &mut local_tt_thread,
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

    pub fn clear_state(&self) {
        NODE_COUNT.store(0, Ordering::Relaxed)
    }
}

// Helper functions like `search_node_with_arena` and `print_uci_info` live in `core.rs`.

// MVV/LVA score: higher is better. Use victim material scaled minus attacker material.
// quiescence, ordering and helpers live in `crate::search::quiescence` to keep this file focused.

// Capture generator moved into `bitboard::movegen::generate_pseudo_captures`.
