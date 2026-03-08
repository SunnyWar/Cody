// src/search/search.rs

use crate::core::arena::Arena;
use crate::search::core::INF;
use crate::search::core::MATE_SCORE;
use crate::search::core::MAX_REPETITION_HISTORY;
use crate::search::core::NODE_COUNT;
use crate::search::core::RepetitionState;
use crate::search::core::SearchContext;
use crate::search::core::SearchHeuristics;
use crate::search::core::SearchWindow;
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
use std::cell::RefCell;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering;
use std::time::Instant;

const ASPIRATION_START_DELTA_CP: i32 = 25;
const ASPIRATION_MAX_RESEARCHES: usize = 4;
const ASPIRATION_MIN_DEPTH: usize = 3;
const ASPIRATION_MATE_GUARD_CP: i32 = 500;
const PARALLEL_MIN_DEPTH: usize = 12; // Only use threading for deep searches

pub struct Engine<
    M: MoveGenerator + Clone + Send + Sync + 'static,
    E: Evaluator + Clone + Send + Sync + 'static,
> {
    arena: Arena,
    movegen: M,
    evaluator: E,
    arena_capacity: usize,
    num_threads: usize,
    tt: Arc<RwLock<crate::core::tt::TranspositionTable>>,
    thread_pool: Option<rayon::ThreadPool>,
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
            tt: Arc::new(RwLock::new(crate::core::tt::TranspositionTable::new(20))),
            thread_pool: None,
        }
    }

    /// Set number of threads to use for root parallelism. 1 = serial.
    pub fn set_num_threads(&mut self, n: usize) {
        let n = n.max(1);
        if self.num_threads != n {
            self.num_threads = n;
            // Rebuild thread pool if threads > 1
            if n > 1 {
                self.thread_pool = rayon::ThreadPoolBuilder::new().num_threads(n).build().ok();
            } else {
                self.thread_pool = None;
            }
        }
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

        self.tt = Arc::new(RwLock::new(crate::core::tt::TranspositionTable::new(
            size_pow2,
        )));
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

            if self.num_threads <= 1 || d < PARALLEL_MIN_DEPTH {
                // Use serial search for single-threaded mode or shallow depths
                // (parallel overhead not worth it for shallow searches)
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
                // Parallel root move evaluation using rayon with persistent thread pool
                use rayon::prelude::*;

                // Use the persistent thread pool (already created in set_num_threads)
                let pool = self
                    .thread_pool
                    .as_ref()
                    .expect("Thread pool not initialized");

                // Clone components into the closure so each thread owns its data.
                let mg = self.movegen.clone();
                let ev = self.evaluator.clone();
                let arena_cap = self.arena_capacity;

                // Shared atomic alpha for basic cutoffs across threads
                // Use -INF instead of i32::MIN to avoid overflow when negating values
                let shared_alpha = Arc::new(AtomicI32::new(-INF));

                // Use thread-local storage to reuse arenas and TTs across moves
                thread_local! {
                    static THREAD_ARENA: RefCell<Option<Arena>> = const { RefCell::new(None) };
                    static THREAD_TT: RefCell<Option<crate::core::tt::TranspositionTable>> = const { RefCell::new(None) };
                }

                let results: Vec<(ChessMove, i32)> = pool.install(|| {
                    moves
                        .as_slice()
                        .par_iter()
                        .copied()
                        .map(move |m| {
                            // Get or create thread-local arena (reused across all moves in this
                            // thread)
                            let score = THREAD_ARENA.with(|arena_cell| {
                                let mut arena_opt = arena_cell.borrow_mut();
                                if arena_opt.is_none() {
                                    *arena_opt = Some(Arena::new(arena_cap));
                                }
                                let local_arena = arena_opt.as_mut().unwrap();

                                local_arena.get_mut(0).position.copy_from(root);
                                {
                                    let (parent, child) = local_arena.get_pair_mut(0, 1);
                                    parent.position.apply_move_into(&m, &mut child.position);
                                }

                                let mut repetition_history = [0u64; MAX_REPETITION_HISTORY];
                                repetition_history[0] = root.zobrist_hash();
                                repetition_history[1] = local_arena.get(1).position.zobrist_hash();

                                // Get or create thread-local TT (reused across all moves in this
                                // thread)
                                THREAD_TT.with(|tt_cell| {
                                    let mut tt_opt = tt_cell.borrow_mut();
                                    if tt_opt.is_none() {
                                        // 16MB per thread (2^20 entries)
                                        *tt_opt =
                                            Some(crate::core::tt::TranspositionTable::new(20));
                                    }
                                    let local_tt = tt_opt.as_mut().unwrap();
                                    let mut local_heuristics = SearchHeuristics::new();

                                    // Use shared alpha for better cutoffs in parallel search
                                    let current_alpha = shared_alpha.load(Ordering::Relaxed);

                                    let mut ctx = SearchContext {
                                        movegen: &mg,
                                        evaluator: &ev,
                                        tt: local_tt,
                                        heuristics: &mut local_heuristics,
                                        stop,
                                        time_budget_ms,
                                        start_time: Some(&start),
                                    };

                                    let score = -search_node_with_arena(
                                        &mut ctx,
                                        local_arena,
                                        1,
                                        d - 1,
                                        &mut SearchWindow {
                                            alpha: -INF,
                                            beta: -current_alpha,
                                        },
                                        &mut RepetitionState {
                                            history: repetition_history,
                                            len: 2,
                                        },
                                    );

                                    // Update shared alpha if we found a better move
                                    loop {
                                        let current = shared_alpha.load(Ordering::Relaxed);
                                        if score <= current {
                                            break;
                                        }
                                        if shared_alpha
                                            .compare_exchange(
                                                current,
                                                score,
                                                Ordering::Relaxed,
                                                Ordering::Relaxed,
                                            )
                                            .is_ok()
                                        {
                                            break;
                                        }
                                    }

                                    score
                                })
                            });
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
            let pv_str = if crate::VERBOSE.load(Ordering::Relaxed) && !last_completed_move.is_null()
            {
                last_completed_move.to_string()
            } else {
                "".to_string()
            };
            let seldepth = current_seldepth().max(d);
            let hashfull = self.tt.read().unwrap().hashfull_per_mille();
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
        let key = self.arena.get(0).position.zobrist_hash();
        let tt_guard = self.tt.read().unwrap();
        if let Some(e) = tt_guard.probe(key, d as i8, -INF, INF) {
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
        self.tt.write().unwrap().clear();
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
        // Serial path: get exclusive access to the TT
        let mut tt_guard = self.tt.write().unwrap();
        let tt_ref: &mut crate::core::tt::TranspositionTable = &mut tt_guard;

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

            let mut ctx = SearchContext {
                movegen: &self.movegen,
                evaluator: &self.evaluator,
                tt: tt_ref,
                heuristics,
                stop,
                time_budget_ms,
                start_time: Some(start),
            };

            let score = if i == 0 {
                let mut window = SearchWindow {
                    alpha: -beta,
                    beta: -local_alpha,
                };
                let mut rep = RepetitionState {
                    history: repetition_history,
                    len: 2,
                };
                -search_node_with_arena(&mut ctx, &mut self.arena, 1, d - 1, &mut window, &mut rep)
            } else {
                let mut window = SearchWindow {
                    alpha: -local_alpha - 1,
                    beta: -local_alpha,
                };
                let mut rep = RepetitionState {
                    history: repetition_history,
                    len: 2,
                };
                let mut pvs_score = -search_node_with_arena(
                    &mut ctx,
                    &mut self.arena,
                    1,
                    d - 1,
                    &mut window,
                    &mut rep,
                );

                if pvs_score > local_alpha && pvs_score < beta {
                    let mut full_window = SearchWindow {
                        alpha: -beta,
                        beta: -local_alpha,
                    };
                    let mut full_rep = RepetitionState {
                        history: repetition_history,
                        len: 2,
                    };
                    pvs_score = -search_node_with_arena(
                        &mut ctx,
                        &mut self.arena,
                        1,
                        d - 1,
                        &mut full_window,
                        &mut full_rep,
                    );
                }

                pvs_score
            };

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
                let pv_str = if crate::VERBOSE.load(Ordering::Relaxed) && !best_move.is_null() {
                    best_move.to_string()
                } else {
                    "".to_string()
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
