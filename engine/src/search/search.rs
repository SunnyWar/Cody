// src/search/search.rs

use crate::core::arena::Arena;
use crate::search::core::{INF, MATE_SCORE, NODE_COUNT, print_uci_info, search_node_with_arena};
use crate::search::evaluator::Evaluator;
use bitboard::Square;
use bitboard::{
    mov::ChessMove,
    movegen::{MoveGenerator, generate_legal_moves},
    position::Position,
};
use rustc_hash::FxHashMap;
use std::sync::atomic::Ordering;
use std::time::Instant;

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

    /// Iterative deepening search. max_depth is the maximum search depth to perform.
    /// Optionally accepts a time budget in milliseconds and a stop flag reference. If the
    /// time budget is provided the search will stop after completing the last fully
    /// finished depth that doesn't exceed the budget. The stop flag (AtomicBool) can be
    /// used by the caller to request an early stop; if provided the search will check it
    /// between completed depths.
    pub fn search(
        &mut self,
        root: &Position,
        max_depth: usize,
        time_budget_ms: Option<u64>,
        stop: Option<&std::sync::atomic::AtomicBool>,
    ) -> (ChessMove, i32) {
        // Track the overall start time for nps calculations
        let start = Instant::now();

        let mut last_completed_move = ChessMove::null();
        let mut last_completed_score = i32::MIN;

        // Iterative deepening loop
        for d in 1..=max_depth {
            // Prepare arena and root position for this depth
            self.arena.reset();
            self.arena.get_mut(0).position.copy_from(root);

            let mut moves = {
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

            // Build a (from,to) → index map once
            let move_index: FxHashMap<(Square, Square), usize> = moves
                .iter()
                .enumerate()
                .map(|(i, m)| ((m.from, m.to), i))
                .collect();

            // Probe TT and reorder instantly if match found
            self.probe_for_best_move(d, &mut moves, &move_index);

            if self.num_threads <= 1 {
                // Serial path: prepare a TT reference that points at our engine TT if present,
                // otherwise to a temporary local table used only for this search.
                let mut local_tt_storage;
                let tt_ref: &mut crate::core::tt::TranspositionTable = match self.tt.as_mut() {
                    Some(ref_mut) => ref_mut,
                    None => {
                        local_tt_storage = crate::core::tt::TranspositionTable::new(1);
                        &mut local_tt_storage
                    }
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
                        d - 1,
                        -INF,
                        INF,
                        tt_ref,
                    );

                    if score > best_score {
                        best_score = score;
                        best_move = m;
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
                            // Each thread gets its own TT instance (no shared access)
                            let mut local_tt_thread = crate::core::tt::TranspositionTable::new(1);
                            let score = -search_node_with_arena(
                                &mg,
                                &ev,
                                &mut local_arena,
                                1,
                                d - 1,
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
                    }
                }
            }

            // Completed this depth successfully; compute elapsed and print UCI info.
            last_completed_move = best_move;
            last_completed_score = best_score;
            let elapsed = start.elapsed().as_millis() as u64;
            let pv_str = if last_completed_move.is_null() {
                "".to_string()
            } else {
                last_completed_move.to_string()
            };
            //print_uci_info(d, last_completed_score, &pv_str, elapsed);

            // Stop if time budget exceeded or external stop requested
            if let Some(mt) = time_budget_ms
                && elapsed >= mt
            {
                break;
            }
            if let Some(stopflag) = stop
                && stopflag.load(Ordering::Relaxed)
            {
                break;
            }
        }

        (last_completed_move, last_completed_score)
    }

    fn probe_for_best_move(
        &mut self,
        d: usize,
        moves: &mut Vec<ChessMove>,
        move_index: &FxHashMap<(Square, Square), usize>,
    ) {
        let Some(ttref) = self.tt.as_ref() else {
            return;
        };

        let key = self.arena.get(0).position.zobrist_hash();
        if let Some(e) = ttref.probe(key, d as i8, -INF, INF) {
            let bmove = e.best_move;
            if !bmove.is_null() {
                if let Some(&pos) = move_index.get(&(bmove.from, bmove.to)) {
                    if pos != 0 {
                        moves.swap(0, pos);
                    }
                }
            }
        }
    }

    pub fn clear_state(&mut self) {
        NODE_COUNT.store(0, Ordering::Relaxed);
        if let Some(tt) = self.tt.as_mut() {
            tt.clear();
        }
    }
}
