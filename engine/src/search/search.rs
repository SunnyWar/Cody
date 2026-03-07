// src/search/search.rs

use crate::core::arena::Arena;
use crate::search::core::INF;
use crate::search::core::MATE_SCORE;
use crate::search::core::MAX_REPETITION_HISTORY;
use crate::search::core::NODE_COUNT;
use crate::search::core::SearchHeuristics;
use crate::search::core::current_seldepth;
use crate::search::core::order_moves_with_heuristics_fast;
use crate::search::core::reset_seldepth;
use crate::search::core::search_node_with_arena;
use crate::search::evaluator::Evaluator;
use crate::search::evaluator::evaluate_for_side_to_move;
use bitboard::MoveList;
use bitboard::mov::ChessMove;
use bitboard::movegen::MoveGenerator;
use bitboard::movegen::generate_legal_moves_fast;
use bitboard::position::Position;
use std::sync::atomic::Ordering;
use std::time::Instant;

const ASPIRATION_START_DELTA_CP: i32 = 25;
const ASPIRATION_MAX_RESEARCHES: usize = 4;
const ASPIRATION_MIN_DEPTH: usize = 3;
const ASPIRATION_MATE_GUARD_CP: i32 = 500;

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

    /// Set the hash table size in megabytes. The actual size will be
    /// rounded to the nearest power of 2 for efficient indexing.
    /// Size is clamped between 1 MB and 1024 MB.
    pub fn set_hash_size_mb(&mut self, size_mb: usize) {
        let size_mb = size_mb.clamp(1, 1024);
        // Each TTEntry is approximately 24 bytes
        let entry_size = std::mem::size_of::<crate::core::tt::TTEntry>();
        let target_entries = (size_mb * 1024 * 1024) / entry_size;

        // Find nearest power of 2
        let mut size_pow2 = 1;
        while (1usize << size_pow2) < target_entries && size_pow2 < 30 {
            size_pow2 += 1;
        }

        self.tt = Some(crate::core::tt::TranspositionTable::new(size_pow2));
    }

    /// Iterative deepening search. max_depth is the maximum search depth to
    /// perform. Optionally accepts a time budget in milliseconds and a stop
    /// flag reference. If the time budget is provided the search will stop
    /// after completing the last fully finished depth that doesn't exceed
    /// the budget. The stop flag (AtomicBool) can be used by the caller to
    /// request an early stop; if provided the search will check it
    /// between completed depths.
    pub fn search(
        &mut self,
        root: &Position,
        max_depth: usize,
        time_budget_ms: Option<u64>,
        stop: Option<&std::sync::atomic::AtomicBool>,
    ) -> (ChessMove, i32) {
        if max_depth == 0 {
            let moves = generate_legal_moves_fast(root);
            if moves.is_empty() {
                let score = if self.movegen.in_check(root) {
                    -MATE_SCORE
                } else {
                    0
                };
                return (ChessMove::null(), score);
            }

            // Even at depth 0, never emit 0000 from a non-terminal position.
            return (moves[0], evaluate_for_side_to_move(&self.evaluator, root));
        }

        // Track the overall start time for nps calculations
        let start = Instant::now();
        let mut last_info_time = start;

        let mut last_completed_move = ChessMove::null();
        let mut last_completed_score = i32::MIN;
        let mut heuristics = SearchHeuristics::new();

        // Iterative deepening loop
        for d in 1..=max_depth {
            reset_seldepth(0);

            // Prepare arena and root position for this depth
            self.arena.reset();
            self.arena.get_mut(0).position.copy_from(root);

            let mut moves = {
                let (parent, _) = self.arena.get_pair_mut(0, 1);
                generate_legal_moves_fast(&parent.position)
            };

            // Diagnostic movegen validation is expensive; keep it for debug
            // sessions only when verbose logging is enabled.
            #[cfg(debug_assertions)]
            if crate::VERBOSE.load(Ordering::Relaxed) {
                bitboard::movegen::validate_legal_move_generation(&self.arena.get(0).position);
            }

            if moves.is_empty() {
                let score = if self.movegen.in_check(root) {
                    -MATE_SCORE
                } else {
                    0
                };
                return (ChessMove::null(), score);
            }

            let fallback_move = moves[0];
            let mut best_score = i32::MIN;
            let mut best_move = fallback_move;
            let mut searched_any = false;

            // Probe TT and reorder instantly if match found
            self.probe_for_best_move(d, &mut moves);
            order_moves_with_heuristics_fast(
                root,
                &mut moves,
                &heuristics,
                0,
                (!last_completed_move.is_null()).then_some(last_completed_move),
            );

            if self.num_threads <= 1 {
                let can_use_aspiration = d >= ASPIRATION_MIN_DEPTH
                    && last_completed_score != i32::MIN
                    && last_completed_score.abs() < MATE_SCORE - ASPIRATION_MATE_GUARD_CP;

                if can_use_aspiration {
                    let mut delta = ASPIRATION_START_DELTA_CP;
                    let mut alpha = (last_completed_score - delta).max(-INF);
                    let mut beta = (last_completed_score + delta).min(INF);
                    let mut researches = 0usize;

                    loop {
                        let (window_best_move, window_best_score, window_searched_any) = self
                            .search_root_serial_window(
                                root,
                                d,
                                &moves,
                                time_budget_ms,
                                stop,
                                &start,
                                &mut last_info_time,
                                &mut heuristics,
                                alpha,
                                beta,
                            );

                        best_move = window_best_move;
                        best_score = window_best_score;
                        searched_any = window_searched_any;

                        // On timeout/stop before any move, keep previous completed result.
                        if !searched_any {
                            break;
                        }

                        let fail_low = best_score <= alpha;
                        let fail_high = best_score >= beta;
                        if !fail_low && !fail_high {
                            break;
                        }

                        researches += 1;
                        if researches >= ASPIRATION_MAX_RESEARCHES {
                            let (full_best_move, full_best_score, full_searched_any) = self
                                .search_root_serial_window(
                                    root,
                                    d,
                                    &moves,
                                    time_budget_ms,
                                    stop,
                                    &start,
                                    &mut last_info_time,
                                    &mut heuristics,
                                    -INF,
                                    INF,
                                );
                            best_move = full_best_move;
                            best_score = full_best_score;
                            searched_any = full_searched_any;
                            break;
                        }

                        delta = delta.saturating_mul(2);
                        if fail_low {
                            alpha = (last_completed_score - delta).max(-INF);
                        }
                        if fail_high {
                            beta = (last_completed_score + delta).min(INF);
                        }
                    }
                } else {
                    (best_move, best_score, searched_any) = self.search_root_serial_window(
                        root,
                        d,
                        &moves,
                        time_budget_ms,
                        stop,
                        &start,
                        &mut last_info_time,
                        &mut heuristics,
                        -INF,
                        INF,
                    );
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
                        .as_slice()
                        .par_iter()
                        .copied()
                        .map(move |m| {
                            // Each thread gets its own arena to avoid synchronization
                            let mut local_arena = Arena::new(arena_cap);
                            local_arena.get_mut(0).position.copy_from(root);
                            {
                                let (parent, child) = local_arena.get_pair_mut(0, 1);
                                parent.position.apply_move_into(&m, &mut child.position);
                            }

                            let mut repetition_history = [0u64; MAX_REPETITION_HISTORY];
                            repetition_history[0] = root.zobrist_hash();
                            repetition_history[1] = local_arena.get(1).position.zobrist_hash();

                            let mut local_tt_thread = crate::core::tt::TranspositionTable::new(1);
                            let mut local_heuristics = SearchHeuristics::new();
                            let score = -search_node_with_arena(
                                &mg,
                                &ev,
                                &mut local_arena,
                                1,
                                d - 1,
                                -INF,
                                INF,
                                &mut local_tt_thread,
                                &mut local_heuristics,
                                stop,
                                time_budget_ms,
                                Some(&start),
                                &mut repetition_history,
                                2,
                            );
                            (m, score)
                        })
                        .collect()
                });

                for (m, score) in results {
                    if !searched_any || score > best_score {
                        best_score = score;
                        best_move = m;
                    }
                    searched_any = true;
                }
            }

            if !searched_any {
                // Time/stop can cut this depth before any root move is searched.
                // Do not overwrite the last completed move with 0000.
                if last_completed_move.is_null() {
                    last_completed_move = fallback_move;
                    last_completed_score = evaluate_for_side_to_move(&self.evaluator, root);
                }
                break;
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
            let seldepth = current_seldepth().max(d);
            let hashfull = self
                .tt
                .as_ref()
                .map(|tt| tt.hashfull_per_mille())
                .unwrap_or(0);
            // Always print info at the end of each depth
            crate::search::core::print_uci_info(
                d,
                seldepth,
                last_completed_score,
                &pv_str,
                elapsed,
                hashfull,
            );

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

    fn probe_for_best_move(&mut self, d: usize, moves: &mut MoveList) {
        let Some(ttref) = self.tt.as_ref() else {
            return;
        };

        let key = self.arena.get(0).position.zobrist_hash();
        if let Some(e) = ttref.probe(key, d as i8, -INF, INF) {
            let bmove = e.best_move;
            if bmove.is_null() {
                return;
            }

            for i in 0..moves.len() {
                if moves[i] == bmove {
                    if i != 0 {
                        moves.swap(0, i);
                    }
                    return;
                }
            }

            #[cfg(debug_assertions)]
            if crate::VERBOSE.load(Ordering::Relaxed) {
                eprintln!(
                    "[debug] TT best move not found in root move list: move={} key={}",
                    bmove, key
                );
            }
        }
    }

    pub fn clear_state(&mut self) {
        NODE_COUNT.store(0, Ordering::Relaxed);
        if let Some(tt) = self.tt.as_mut() {
            tt.clear();
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn search_root_serial_window(
        &mut self,
        root: &Position,
        d: usize,
        moves: &MoveList,
        time_budget_ms: Option<u64>,
        stop: Option<&std::sync::atomic::AtomicBool>,
        start: &Instant,
        last_info_time: &mut Instant,
        heuristics: &mut SearchHeuristics,
        alpha: i32,
        beta: i32,
    ) -> (ChessMove, i32, bool) {
        // Serial path: prefer engine TT if configured, otherwise use a tiny
        // temporary table for this pass.
        let mut local_tt_storage;
        let tt_ref: &mut crate::core::tt::TranspositionTable = match self.tt.as_mut() {
            Some(ref_mut) => ref_mut,
            None => {
                local_tt_storage = crate::core::tt::TranspositionTable::new(1);
                &mut local_tt_storage
            }
        };

        let mut best_score = i32::MIN;
        let mut best_move = moves[0];
        let mut searched_any = false;
        let mut local_alpha = alpha;

        for i in 0..moves.len() {
            let m = moves[i];
            // Check stop flag and time budget before each root move.
            let now = Instant::now();
            let elapsed = now.duration_since(*start).as_millis() as u64;
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

            {
                let (parent, child) = self.arena.get_pair_mut(0, 1);
                parent.position.apply_move_into(&m, &mut child.position);
            }

            let mut repetition_history = [0u64; MAX_REPETITION_HISTORY];
            repetition_history[0] = root.zobrist_hash();
            repetition_history[1] = self.arena.get(1).position.zobrist_hash();

            let score = -search_node_with_arena(
                &self.movegen,
                &self.evaluator,
                &mut self.arena,
                1,
                d - 1,
                -beta,
                -local_alpha,
                tt_ref,
                heuristics,
                stop,
                time_budget_ms,
                Some(start),
                &mut repetition_history,
                2,
            );

            if !searched_any || score > best_score {
                best_score = score;
                best_move = m;
            }
            searched_any = true;

            if score > local_alpha {
                local_alpha = score;
            }
            if local_alpha >= beta {
                break;
            }

            // Periodic progress info is useful for timed UCI searches,
            // but it is expensive noise for fixed-depth bench runs.
            if time_budget_ms.is_some() && now.duration_since(*last_info_time).as_millis() >= 1000 {
                let pv_str = if best_move.is_null() {
                    "".to_string()
                } else {
                    best_move.to_string()
                };
                let seldepth = current_seldepth().max(d);
                let hashfull = tt_ref.hashfull_per_mille();
                crate::search::core::print_uci_info(
                    d, seldepth, best_score, &pv_str, elapsed, hashfull,
                );
                *last_info_time = now;
            }
        }

        (best_move, best_score, searched_any)
    }
}
