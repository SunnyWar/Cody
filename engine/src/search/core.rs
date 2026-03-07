use crate::VERBOSE;
use crate::core::arena::Arena;
use crate::core::tt::TTFlag;
use crate::core::tt::TranspositionTable;
use crate::search::evaluator::Evaluator;
use crate::search::quiescence::quiescence_with_arena;
use crate::util;
use bitboard::mov::ChessMove;
use bitboard::mov::MoveType;
use bitboard::movegen::MoveGenerator;
use bitboard::movegen::generate_pseudo_moves;
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
const MAX_SEARCH_PLY: usize = 128;
pub const MAX_REPETITION_HISTORY: usize = MAX_SEARCH_PLY + 4;

#[inline]
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

fn piece_value(kind: PieceKind) -> i32 {
    match kind {
        PieceKind::Pawn => 100,
        PieceKind::Knight => 320,
        PieceKind::Bishop => 330,
        PieceKind::Rook => 500,
        PieceKind::Queen => 900,
        PieceKind::King => 10_000,
    }
}

fn get_piece_on_square(pos: &bitboard::position::Position, sq: bitboard::Square) -> Option<Piece> {
    let mask = bitboard::BitBoardMask::from_square(sq);
    for (piece, bb) in pos.pieces.iter() {
        if (bb & mask).is_nonempty() {
            return Some(piece);
        }
    }
    None
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

    let victim_value = victim_piece.map(|p| piece_value(p.kind())).unwrap_or(0);
    let attacker_piece = get_piece_on_square(pos, mv.from);
    let attacker_value = attacker_piece.map(|p| piece_value(p.kind())).unwrap_or(0);
    victim_value * 100 - attacker_value
}

#[inline(always)]
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
             0 time {} pv {}",
            depth, seldepth, mate_in, nodes, nps, hashfull, elapsed_ms, pv
        )
    } else {
        format!(
            "info depth {} seldepth {} multipv 1 score cp {} nodes {} nps {} hashfull {} tbhits 0 \
             time {} pv {}",
            depth, seldepth, score, nodes, nps, hashfull, elapsed_ms, pv
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

// Helper recursive search that operates on a provided arena and components.

pub fn search_node_with_arena<M: MoveGenerator, E: Evaluator>(
    movegen: &M,
    evaluator: &E,
    arena: &mut Arena,
    ply: usize,
    remaining: usize,
    mut alpha: i32,
    beta: i32,
    tt: &mut TranspositionTable,
    heuristics: &mut SearchHeuristics,
    stop: Option<&std::sync::atomic::AtomicBool>,
    time_budget_ms: Option<u64>,
    start_time: Option<&std::time::Instant>,
    repetition_history: &mut [u64; MAX_REPETITION_HISTORY],
    repetition_len: usize,
) -> i32 {
    NODE_COUNT.fetch_add(1, Ordering::Relaxed);
    update_seldepth(ply);
    let original_alpha = alpha;
    // Check stop flag and time budget at each node
    if let Some(stopflag) = stop
        && stopflag.load(Ordering::Relaxed)
    {
        return 0;
    }

    if let (Some(mt), Some(start)) = (time_budget_ms, start_time) {
        let elapsed = start.elapsed().as_millis() as u64;
        if elapsed >= mt {
            return 0;
        }
    }

    if remaining == 0 {
        return quiescence_with_arena(movegen, evaluator, arena, ply, alpha, beta);
    }

    // Compute key once; full Zobrist recomputation is expensive.
    let key = arena.get(ply).position.zobrist_hash();

    // Draw adjudication.
    // 1) Threefold repetition from the current search path.
    if is_threefold_repetition(key, repetition_history, repetition_len) {
        return 0;
    }

    // 2) Fifty-move rule (claimable draw). We treat claimable draws as
    // immediate draws to avoid wasting search on objectively drawn lines.
    if arena.get(ply).position.halfmove_clock >= 100 {
        return 0;
    }

    // Probe TT if provided (tt is always present in serial path; for parallel we
    // pass a local dummy).
    // - Exact entries with a non-null move are verified against the generated legal
    //   move list before being trusted.
    // - Lower/Upper entries returned by `probe` are already window-validated and
    //   can be used as immediate cutoffs.
    let mut tt_exact_needs_verify: Option<crate::core::tt::TTEntry> = None;
    {
        if let Some(e) = tt.probe(key, remaining as i8, alpha, beta) {
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
    // qsearch collision), (c) not root.
    let pos_ref = arena.get(ply).position;
    if ply > 0 && remaining > 2 && !movegen.in_check(&pos_ref) {
        // Make a null move (pass)
        let mut child_pos = pos_ref;
        child_pos.side_to_move = pos_ref.side_to_move.opposite();
        child_pos.ep_square = None;

        // Store and restore for non-destructive probe
        if ply + 1 < MAX_SEARCH_PLY {
            arena.get_mut(ply + 1).position = child_pos;
            let null_reduction = (remaining / 3).max(1);
            let child_key = arena.get(ply + 1).position.zobrist_hash();
            let next_rep_len = if repetition_len < MAX_REPETITION_HISTORY {
                repetition_history[repetition_len] = child_key;
                repetition_len + 1
            } else {
                repetition_len
            };
            let null_score = -search_node_with_arena(
                movegen,
                evaluator,
                arena,
                ply + 1,
                remaining - null_reduction - 1,
                -beta,
                -beta + 1,
                tt,
                heuristics,
                stop,
                time_budget_ms,
                start_time,
                repetition_history,
                next_rep_len,
            );
            if null_score >= beta {
                return null_score;
            }
        }
    }

    let mut moves = {
        let (parent, _) = arena.get_pair_mut(ply, ply + 1);
        generate_pseudo_moves(&parent.position)
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
    let mut best_move = bitboard::mov::ChessMove::null();
    // Work with a local mutable vector so we can reorder based on TT best move
    // Prioritize TT best move first (already at 0 if found).
    if let Some(e) = tt_exact_needs_verify
        && !e.best_move.is_null()
        && let Some(idx) = moves.iter().position(|m| *m == e.best_move)
        && idx != 0
    {
        moves.swap(0, idx);
    }

    let pos = arena.get(ply).position;
    // Full-list move ordering gives alpha-beta the best chance to cut early.
    order_moves_with_heuristics(&pos, &mut moves, heuristics, ply, None);

    let moves_vec = moves;
    if let Some(e) = tt_exact_needs_verify
        && !e.best_move.is_null()
        && moves_vec.contains(&e.best_move)
    {
        let mut tt_move_is_legal = false;
        {
            let (parent, child) = arena.get_pair_mut(ply, ply + 1);
            parent
                .position
                .apply_move_into(&e.best_move, &mut child.position);
            if !mover_left_in_check(movegen, &parent.position, &child.position) {
                tt_move_is_legal = true;
            }
        }
        if tt_move_is_legal {
            return e.value;
        }
    }

    let mut legal_move_count = 0usize;
    for m in moves_vec.iter().cloned() {
        {
            let (parent, child) = arena.get_pair_mut(ply, ply + 1);
            parent.position.apply_move_into(&m, &mut child.position);

            if mover_left_in_check(movegen, &parent.position, &child.position) {
                continue;
            }
        }

        let move_index = legal_move_count;
        legal_move_count += 1;

        let child_key = arena.get(ply + 1).position.zobrist_hash();
        let next_rep_len = if repetition_len < MAX_REPETITION_HISTORY {
            repetition_history[repetition_len] = child_key;
            repetition_len + 1
        } else {
            repetition_len
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

        // Search with reduced (or normal) depth
        let mut score = -search_node_with_arena(
            movegen,
            evaluator,
            arena,
            ply + 1,
            depth_for_search,
            -beta,
            -alpha,
            tt,
            heuristics,
            stop,
            time_budget_ms,
            start_time,
            repetition_history,
            next_rep_len,
        );

        // If LMR returned a value > alpha, re-search at full depth to verify
        if !do_full_depth_search && score > alpha {
            score = -search_node_with_arena(
                movegen,
                evaluator,
                arena,
                ply + 1,
                remaining - 1,
                -beta,
                -alpha,
                tt,
                heuristics,
                stop,
                time_budget_ms,
                start_time,
                repetition_history,
                next_rep_len,
            );
        }

        if score > best_score {
            best_score = score;
            best_move = m;
        }

        if score > alpha {
            alpha = score;
        }

        // Beta cutoff
        if alpha >= beta {
            heuristics.update_on_beta_cutoff(ply, m, remaining);
            break;
        }
    }

    if legal_move_count == 0 {
        let pos = &arena.get(ply).position;
        if movegen.in_check(pos) {
            return -MATE_SCORE + ply as i32;
        }
        return 0;
    }

    // Store TT result with correct bound semantics from the original window.
    let tt_flag = if best_score <= original_alpha {
        TTFlag::Upper
    } else if best_score >= beta {
        TTFlag::Lower
    } else {
        TTFlag::Exact
    };
    tt.store(key, best_score, remaining as i8, tt_flag, best_move);

    best_score
}
