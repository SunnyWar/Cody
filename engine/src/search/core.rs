use crate::VERBOSE;
use crate::core::arena::Arena;
use crate::core::tt::TTFlag;
use crate::core::tt::TranspositionTable;
use crate::search::evaluator::Evaluator;
use crate::search::evaluator::evaluate_for_side_to_move;
use crate::search::quiescence::quiescence_with_arena;
use crate::util;
use bitboard::MoveList;
use bitboard::mov::ChessMove;
use bitboard::mov::MoveType;
use bitboard::movegen::MoveGenerator;
use bitboard::movegen::generate_pseudo_moves_fast;
use bitboard::piece::Color;
use bitboard::piece::Piece;
use bitboard::piece::PieceKind;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

pub static NODE_COUNT: AtomicU64 = AtomicU64::new(0);
pub static SELDEPTH_MAX: AtomicUsize = AtomicUsize::new(0);
pub static TB_HITS: AtomicU64 = AtomicU64::new(0);

pub fn reset_seldepth(initial_ply: usize) {
    SELDEPTH_MAX.store(initial_ply, Ordering::Relaxed);
}

pub fn update_seldepth(ply: usize) {
    SELDEPTH_MAX.fetch_max(ply, Ordering::Relaxed);
}

pub fn current_seldepth() -> usize {
    SELDEPTH_MAX.load(Ordering::Relaxed)
}

// Positive large value used to detect mate scores. Keep consistent with UCI
// API's MATE_SCORE.
pub const MATE_SCORE: i32 = 30_000;
// Large infinity value for alpha-beta bounds
pub const INF: i32 = 1_000_000_000;
const NULL_MOVE_STATIC_MARGIN_CP: i32 = 120;
const MAX_SEARCH_PLY: usize = 128;
pub const MAX_REPETITION_HISTORY: usize = MAX_SEARCH_PLY + 4;

fn is_threefold_repetition(
    key: u64,
    repetition_history: &[u64; MAX_REPETITION_HISTORY],
    repetition_len: usize,
) -> bool {
    let mut count = 0usize;
    for seen in repetition_history.iter().take(repetition_len) {
        if *seen == key {
            count += 1;
            if count >= 3 {
                return true;
            }
        }
    }
    false
}

pub struct SearchHeuristics {
    killer_moves: [[ChessMove; 2]; MAX_SEARCH_PLY],
    history: [[i32; 64]; 64],
}

impl Default for SearchHeuristics {
    fn default() -> Self {
        Self {
            killer_moves: [[ChessMove::null(); 2]; MAX_SEARCH_PLY],
            history: [[0; 64]; 64],
        }
    }
}

impl SearchHeuristics {
    pub fn new() -> Self {
        Self::default()
    }

    fn score_move(&self, pos: &bitboard::position::Position, mv: &ChessMove, ply: usize) -> i32 {
        let mut score = 0;

        // Tactical moves first.
        score += match mv.move_type {
            MoveType::Capture | MoveType::EnPassant => 100_000 + mvv_lva_score(pos, mv),
            MoveType::Promotion(kind) => {
                90_000
                    + match kind {
                        PieceKind::Queen => 900,
                        PieceKind::Rook => 500,
                        PieceKind::Bishop => 330,
                        PieceKind::Knight => 320,
                        _ => 0,
                    }
            }
            _ => 0,
        };

        // Then killer moves for quiet move ordering.
        if ply < MAX_SEARCH_PLY {
            if self.killer_moves[ply][0] == *mv {
                score += 80_000;
            } else if self.killer_moves[ply][1] == *mv {
                score += 70_000;
            }
        }

        // History heuristic for remaining quiet ordering.
        score + self.history[mv.from.index()][mv.to.index()]
    }

    fn update_on_beta_cutoff(&mut self, ply: usize, mv: ChessMove, depth: usize) {
        // Killer/history are most useful for quiet moves.
        if matches!(
            mv.move_type,
            MoveType::Capture | MoveType::EnPassant | MoveType::Promotion(_)
        ) {
            return;
        }

        if ply < MAX_SEARCH_PLY && self.killer_moves[ply][0] != mv {
            self.killer_moves[ply][1] = self.killer_moves[ply][0];
            self.killer_moves[ply][0] = mv;
        }

        let bonus = (depth * depth) as i32;
        let from = mv.from.index();
        let to = mv.to.index();
        self.history[from][to] = (self.history[from][to] + bonus).min(50_000);
    }
}

pub fn order_moves_with_heuristics(
    pos: &bitboard::position::Position,
    moves: &mut [ChessMove],
    heuristics: &SearchHeuristics,
    ply: usize,
    pv_move: Option<ChessMove>,
) {
    if moves.len() <= 1 {
        return;
    }

    if let Some(pv) = pv_move
        && !pv.is_null()
        && let Some(idx) = moves.iter().position(|m| *m == pv)
        && idx != 0
    {
        moves.swap(0, idx);
    }

    let start = if pv_move.is_some_and(|m| !m.is_null() && moves.first().copied() == Some(m)) {
        1
    } else {
        0
    };

    moves[start..].sort_unstable_by_key(|m| -heuristics.score_move(pos, m, ply));
}

/// Fast version that works with MoveList
pub fn order_moves_with_heuristics_fast(
    pos: &bitboard::position::Position,
    moves: &mut MoveList,
    heuristics: &SearchHeuristics,
    ply: usize,
    pv_move: Option<ChessMove>,
) {
    let len = moves.len();
    if len <= 1 {
        return;
    }

    if let Some(pv) = pv_move
        && !pv.is_null()
    {
        for i in 0..len {
            if moves[i] == pv {
                moves.swap(0, i);
                break;
            }
        }
    }

    let start = if pv_move.is_some_and(|m| !m.is_null() && !moves.is_empty() && moves[0] == m) {
        1
    } else {
        0
    };

    if start + 1 >= len {
        return;
    }

    let slice = moves.as_mut_slice();

    // Cache heuristic scores once and perform in-place insertion sort by score
    // (descending). This avoids repeated score recomputation in comparator-based
    // sorting on the hot path.
    let mut scores = [0i32; 256];
    for i in start..len {
        scores[i] = heuristics.score_move(pos, &slice[i], ply);
    }

    for i in (start + 1)..len {
        let key_move = slice[i];
        let key_score = scores[i];
        let mut j = i;

        while j > start && scores[j - 1] < key_score {
            slice[j] = slice[j - 1];
            scores[j] = scores[j - 1];
            j -= 1;
        }

        slice[j] = key_move;
        scores[j] = key_score;
    }
}

const fn piece_value(kind: PieceKind) -> i32 {
    match kind {
        PieceKind::Pawn => 100,
        PieceKind::Knight => 320,
        PieceKind::Bishop => 330,
        PieceKind::Rook => 500,
        PieceKind::Queen => 900,
        PieceKind::King => 10_000,
    }
}

fn get_piece_on_square(pos: &bitboard::position::Position, sq: bitboard::Square) -> Piece {
    pos.piece_at_square(sq)
}

fn mvv_lva_score(pos: &bitboard::position::Position, mv: &ChessMove) -> i32 {
    let victim_piece = match mv.move_type {
        MoveType::EnPassant => {
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

fn mover_left_in_check<M: MoveGenerator>(
    movegen: &M,
    parent: &bitboard::position::Position,
    child: &bitboard::position::Position,
) -> bool {
    let mut legal_test = *child;
    legal_test.side_to_move = parent.side_to_move;
    movegen.in_check(&legal_test)
}

pub fn print_uci_info(
    depth: usize,
    seldepth: usize,
    score: i32,
    pv: &str, // principal variation as a space-separated string
    elapsed_ms: u64,
    hashfull: u16,
) {
    let nodes = NODE_COUNT.load(Ordering::Relaxed);
    let tbhits = TB_HITS.load(Ordering::Relaxed);
    let nps = elapsed_ms
        .checked_div(1)
        .map(|_| {
            if elapsed_ms > 0 {
                nodes as u128 * 1000 / elapsed_ms as u128
            } else {
                0
            }
        })
        .unwrap_or(0);

    // Build the UCI info string (with mate/centipawn formatting) and write to
    // stdout and append the same line to cody_uci.log for traceability.
    let info_line = if score.abs() > MATE_SCORE - 100 {
        let mate_in = if score > 0 {
            (MATE_SCORE - score + 1) / 2
        } else {
            -(MATE_SCORE + score) / 2
        };
        format!(
            "info depth {} seldepth {} multipv 1 score mate {} nodes {} nps {} hashfull {} tbhits \
             {} time {} pv {}",
            depth, seldepth, mate_in, nodes, nps, hashfull, tbhits, elapsed_ms, pv
        )
    } else {
        format!(
            "info depth {} seldepth {} multipv 1 score cp {} nodes {} nps {} hashfull {} tbhits {} \
             time {} pv {}",
            depth, seldepth, score, nodes, nps, hashfull, tbhits, elapsed_ms, pv
        )
    };

    // Write to stdout first
    println!("{}", info_line);

    // File logging is expensive in hot paths; keep it for verbose sessions.
    if VERBOSE.load(Ordering::Relaxed)
        && let Ok(mut f) = OpenOptions::new()
            .create(true)
            .append(true)
            .open("cody_uci.log")
    {
        let stamp = util::iso_stamp_ms();
        let _ = writeln!(f, "{} OUT: {}", stamp, info_line);
    }
}

// Context struct to reduce function parameter count
pub struct SearchContext<'a, M: MoveGenerator, E: Evaluator> {
    pub movegen: &'a M,
    pub evaluator: &'a E,
    pub tt: &'a mut TranspositionTable,
    pub heuristics: &'a mut SearchHeuristics,
    pub stop: Option<&'a std::sync::atomic::AtomicBool>,
    pub time_budget_ms: Option<u64>,
    pub start_time: Option<&'a std::time::Instant>,
}

/// Search window (alpha-beta bounds) for a single search node.
pub struct SearchWindow {
    pub alpha: i32,
    pub beta: i32,
}

/// Repetition history tracking for the current search path.
pub struct RepetitionState {
    pub history: [u64; MAX_REPETITION_HISTORY],
    pub len: usize,
}

// Helper recursive search that operates on a provided arena and components.
pub fn search_node_with_arena<M: MoveGenerator, E: Evaluator>(
    ctx: &mut SearchContext<M, E>,
    arena: &mut Arena,
    ply: usize,
    remaining: usize,
    window: &mut SearchWindow,
    rep_state: &mut RepetitionState,
) -> i32 {
    NODE_COUNT.fetch_add(1, Ordering::Relaxed);
    update_seldepth(ply);
    let original_alpha = window.alpha;
    // Check stop flag and time budget at each node
    if let Some(stopflag) = ctx.stop
        && stopflag.load(Ordering::Relaxed)
    {
        return 0;
    }

    if let (Some(mt), Some(start)) = (ctx.time_budget_ms, ctx.start_time) {
        let elapsed = start.elapsed().as_millis() as u64;
        if elapsed >= mt {
            return 0;
        }
    }

    if remaining == 0 {
        return quiescence_with_arena(
            ctx.movegen,
            ctx.evaluator,
            arena,
            ply,
            window.alpha,
            window.beta,
        );
    }

    // Compute key once; full Zobrist recomputation is expensive.
    let key = arena.get(ply).position.zobrist_hash();

    // Draw adjudication.
    // 1) Threefold repetition from the current search path.
    if is_threefold_repetition(key, &rep_state.history, rep_state.len) {
        return 0;
    }

    // 2) Fifty-move rule (claimable draw). We treat claimable draws as
    // immediate draws to avoid wasting search on objectively drawn lines.
    if arena.get(ply).position.halfmove_clock >= 100 {
        return 0;
    }

    if let Some(tb_score) = crate::search::tablebase::probe_wdl_cp(&arena.get(ply).position) {
        TB_HITS.fetch_add(1, Ordering::Relaxed);
        return tb_score;
    }

    // Probe TT if provided (tt is always present in serial path; for parallel we
    // pass a local dummy).
    // - Exact entries with a non-null move are verified against the generated legal
    //   move list before being trusted.
    // - Lower/Upper entries returned by `probe` are already window-validated and
    //   can be used as immediate cutoffs.
    let mut tt_exact_needs_verify: Option<crate::core::tt::TTEntry> = None;
    {
        if let Some(e) = ctx
            .tt
            .probe(key, remaining as i8, window.alpha, window.beta)
        {
            if e.flag == crate::core::tt::TTFlag::Exact as u8 {
                if e.best_move.is_null() {
                    return e.value;
                }
                tt_exact_needs_verify = Some(e);
            } else {
                return e.value;
            }
        }
    }

    // Null-move pruning: if we can pass and still fail-high, prune the whole
    // subtree. Only try when: (a) not in check, (b) remaining > 2 (to avoid
    // qsearch collision), (c) not root, (d) static eval is already close to
    // beta so a fail-high is plausible.
    let pos_ref = arena.get(ply).position;
    let static_eval = evaluate_for_side_to_move(ctx.evaluator, &pos_ref);
    let can_try_null = static_eval >= window.beta - NULL_MOVE_STATIC_MARGIN_CP;
    if ply > 0 && remaining > 2 && !ctx.movegen.in_check(&pos_ref) && can_try_null {
        // Make a null move (pass)
        let mut child_pos = pos_ref;
        child_pos.side_to_move = pos_ref.side_to_move.opposite();
        child_pos.ep_square = None;

        // Store and restore for non-destructive probe
        if ply + 1 < MAX_SEARCH_PLY {
            arena.get_mut(ply + 1).position = child_pos;
            let null_reduction = (remaining / 3).max(1);
            let child_key = arena.get(ply + 1).position.zobrist_hash();
            // Optimization: Use unchecked access to avoid redundant bounds checks (safe
            // within ply < MAX_SEARCH_PLY)
            let next_rep_len = if rep_state.len < MAX_REPETITION_HISTORY {
                unsafe {
                    *rep_state.history.get_unchecked_mut(rep_state.len) = child_key;
                }
                rep_state.len + 1
            } else {
                rep_state.len
            };
            let mut null_window = SearchWindow {
                alpha: -window.beta,
                beta: -window.beta + 1,
            };
            let mut null_rep = RepetitionState {
                history: rep_state.history,
                len: next_rep_len,
            };
            let null_score = -search_node_with_arena(
                ctx,
                arena,
                ply + 1,
                remaining - null_reduction - 1,
                &mut null_window,
                &mut null_rep,
            );
            if null_score >= window.beta {
                return null_score;
            }
        }
    }

    let mut moves = {
        let (parent, _) = arena.get_pair_mut(ply, ply + 1);
        generate_pseudo_moves_fast(&parent.position)
    };

    if moves.is_empty() {
        let pos = &arena.get(ply).position;
        if ctx.movegen.in_check(pos) {
            // mate: return losing score adjusted by ply (so earlier mate is worse)
            return -MATE_SCORE + ply as i32;
        }
        return 0;
    }

    let mut best_score = i32::MIN;
    let mut best_move = bitboard::mov::ChessMove::null();
    // Work with a local mutable vector so we can reorder based on TT best move.
    // Prioritize TT best move first (already at 0 if found).
    if let Some(e) = tt_exact_needs_verify
        && !e.best_move.is_null()
    {
        for i in 0..moves.len() {
            if moves[i] == e.best_move {
                moves.swap(0, i);
                break;
            }
        }
    }

    let pos = arena.get(ply).position;
    let tt_best_move = tt_exact_needs_verify
        .map(|e| e.best_move)
        .filter(|m| !m.is_null());
    // Full-list move ordering gives alpha-beta the best chance to cut early.
    order_moves_with_heuristics_fast(&pos, &mut moves, ctx.heuristics, ply, tt_best_move);

    if let Some(e) = tt_exact_needs_verify
        && !e.best_move.is_null()
    {
        let mut has_move = false;
        for i in 0..moves.len() {
            if moves[i] == e.best_move {
                has_move = true;
                break;
            }
        }
        if has_move {
            let mut tt_move_is_legal = false;
            {
                let (parent, child) = arena.get_pair_mut(ply, ply + 1);
                parent
                    .position
                    .apply_move_into(&e.best_move, &mut child.position);
                if !mover_left_in_check(ctx.movegen, &parent.position, &child.position) {
                    tt_move_is_legal = true;
                }
            }
            if tt_move_is_legal {
                return e.value;
            }
        }
    }

    let mut legal_move_count = 0usize;
    for move_idx in 0..moves.len() {
        // Prefetch future move entries while iterating the ordered move list.
        // Applying this in search (instead of movegen) targets the true hot loop.
        if move_idx & 7 == 0 {
            moves.prefetch_next_batch(move_idx);
        }

        let m = moves[move_idx];
        {
            let (parent, child) = arena.get_pair_mut(ply, ply + 1);
            parent.position.apply_move_into(&m, &mut child.position);

            if mover_left_in_check(ctx.movegen, &parent.position, &child.position) {
                continue;
            }
        }

        let move_index = legal_move_count;
        legal_move_count += 1;

        let child_key = arena.get(ply + 1).position.zobrist_hash();
        // Optimization: Use unchecked access to avoid redundant bounds checks (safe
        // within ply < MAX_SEARCH_PLY)
        let next_rep_len = if rep_state.len < MAX_REPETITION_HISTORY {
            unsafe {
                *rep_state.history.get_unchecked_mut(rep_state.len) = child_key;
            }
            rep_state.len + 1
        } else {
            rep_state.len
        };

        // Late Move Reduction (LMR): reduce depth for moves beyond the first 2 or 3
        // if they don't look promising. Re-search at full depth if needed.
        let mut depth_for_search = remaining - 1;
        let mut do_full_depth_search = true;

        if move_index > 2
            && remaining >= 3
            && !matches!(
                m.move_type,
                MoveType::Capture | MoveType::EnPassant | MoveType::Promotion(_)
            )
        {
            // Conservative LMR: reduce by log-based formula
            let reduction = ((move_index as f64).ln() * (remaining as f64).ln() * 0.5) as usize;
            if reduction > 0 {
                depth_for_search = (remaining - 1).saturating_sub(reduction);
                do_full_depth_search = false;
            }
        }

        // Principal Variation Search (PVS): first legal move with full window,
        // later moves with null window and full-window re-search on improvement.
        let mut score = if move_index == 0 {
            let mut child_window = SearchWindow {
                alpha: -window.beta,
                beta: -window.alpha,
            };
            let mut child_rep = RepetitionState {
                history: rep_state.history,
                len: next_rep_len,
            };
            -search_node_with_arena(
                ctx,
                arena,
                ply + 1,
                depth_for_search,
                &mut child_window,
                &mut child_rep,
            )
        } else {
            let mut null_window = SearchWindow {
                alpha: -window.alpha - 1,
                beta: -window.alpha,
            };
            let mut null_rep = RepetitionState {
                history: rep_state.history,
                len: next_rep_len,
            };
            let mut pvs_score = -search_node_with_arena(
                ctx,
                arena,
                ply + 1,
                depth_for_search,
                &mut null_window,
                &mut null_rep,
            );

            if pvs_score > window.alpha && pvs_score < window.beta {
                let mut full_window = SearchWindow {
                    alpha: -window.beta,
                    beta: -window.alpha,
                };
                let mut full_rep = RepetitionState {
                    history: rep_state.history,
                    len: next_rep_len,
                };
                pvs_score = -search_node_with_arena(
                    ctx,
                    arena,
                    ply + 1,
                    depth_for_search,
                    &mut full_window,
                    &mut full_rep,
                );
            }

            pvs_score
        };

        // If LMR returned a value > alpha, re-search at full depth to verify
        if !do_full_depth_search && score > window.alpha {
            let mut lmr_window = SearchWindow {
                alpha: -window.beta,
                beta: -window.alpha,
            };
            let mut lmr_rep = RepetitionState {
                history: rep_state.history,
                len: next_rep_len,
            };
            score = -search_node_with_arena(
                ctx,
                arena,
                ply + 1,
                remaining - 1,
                &mut lmr_window,
                &mut lmr_rep,
            );
        }

        if score > best_score {
            best_score = score;
            best_move = m;
        }

        // Optimization: Use max() for alpha updates instead of if-else.
        // Compiler generates cmov instead of conditional branch, better pipeline
        // utilization.
        window.alpha = window.alpha.max(score);

        // Beta cutoff
        if window.alpha >= window.beta {
            ctx.heuristics.update_on_beta_cutoff(ply, m, remaining);
            break;
        }
    }

    if legal_move_count == 0 {
        let pos = &arena.get(ply).position;
        if ctx.movegen.in_check(pos) {
            return -MATE_SCORE + ply as i32;
        }
        return 0;
    }

    // Store TT result with correct bound semantics from the original window.
    let tt_flag = if best_score <= original_alpha {
        TTFlag::Upper
    } else if best_score >= window.beta {
        TTFlag::Lower
    } else {
        TTFlag::Exact
    };
    ctx.tt
        .store(key, best_score, remaining as i8, tt_flag, best_move);

    best_score
}
