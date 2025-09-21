use crate::VERBOSE;
use crate::core::arena::Arena;
use crate::core::tt::{TTFlag, TranspositionTable};
use crate::search::evaluator::Evaluator;
use crate::search::quiescence::quiescence_with_arena;
use bitboard::movegen::{MoveGenerator, generate_legal_moves};
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};

pub static NODE_COUNT: AtomicU64 = AtomicU64::new(0);

// Positive large value used to detect mate scores. Keep consistent with UCI API's MATE_SCORE.
pub const MATE_SCORE: i32 = 30_000;
// Large infinity value for alpha-beta bounds
pub const INF: i32 = 1_000_000_000;

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

    // Build the UCI info string (with mate/centipawn formatting) and write to stdout
    // and append the same line to cody_uci.log for traceability.
    let info_line = if score.abs() > MATE_SCORE - 100 {
        let mate_in = if score > 0 {
            (MATE_SCORE - score + 1) / 2
        } else {
            -(MATE_SCORE + score) / 2
        };
        format!(
            "info depth {} score mate {} nodes {} time {} nps {} pv {}",
            depth, mate_in, nodes, elapsed_ms, nps, pv
        )
    } else {
        format!(
            "info depth {} score cp {} nodes {} time {} nps {} pv {}",
            depth, score, nodes, elapsed_ms, nps, pv
        )
    };

    // Write to stdout first
    println!("{}", info_line);
    io::stdout().flush().ok();

    // Append to cody_uci.log (best-effort)
    if let Ok(mut f) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("cody_uci.log")
    {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
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
) -> i32 {
    NODE_COUNT.fetch_add(1, Ordering::Relaxed);

    // Debug: announce entry to this node
    if ply <= 2 || NODE_COUNT.load(Ordering::Relaxed) % 500_000 == 0 {
        if VERBOSE.load(Ordering::Relaxed) {
            eprintln!(
                "[debug] search_node enter ply={} remaining={} nodecount={}",
                ply,
                remaining,
                NODE_COUNT.load(Ordering::Relaxed)
            );
        }
    }

    if remaining == 0 {
        return quiescence_with_arena(movegen, evaluator, arena, ply, alpha, beta);
    }

    // Probe TT if provided (tt is always present in serial path; for parallel we pass a local dummy)
    let mut tt_entry_opt: Option<crate::core::tt::TTEntry> = None;
    {
        let key = arena.get(ply).position.zobrist_hash();
        if let Some(e) = tt.probe(key, remaining as i8, alpha, beta) {
            tt_entry_opt = Some(e);
            if e.flag == crate::core::tt::TTFlag::Exact as u8 {
                return e.value;
            }
        }
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
    // Work with a local mutable vector so we can reorder based on TT best move
    let mut moves_vec = moves;
    if let Some(e) = tt_entry_opt {
        let bmove = e.best_move;
        if !bmove.is_null() {
            if let Some(pos) = moves_vec
                .iter()
                .position(|mm| mm.from == bmove.from && mm.to == bmove.to)
            {
                moves_vec.swap(0, pos);
            }
        }
    }

    for m in moves_vec.iter().cloned() {
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
            tt,
        );

        if score > best_score {
            best_score = score;
        }

        if score > alpha {
            alpha = score;
        }

        // Beta cutoff
        if alpha >= beta {
            // store upper bound in TT if available
            {
                let key = arena.get(ply).position.zobrist_hash();
                tt.store(key, alpha, remaining as i8, TTFlag::Upper, m);
            }
            break;
        }
    }

    // store final result in TT as exact
    {
        let key = arena.get(ply).position.zobrist_hash();
        tt.store(
            key,
            best_score,
            remaining as i8,
            TTFlag::Exact,
            bitboard::mov::ChessMove::null(),
        );
    }

    best_score
}
