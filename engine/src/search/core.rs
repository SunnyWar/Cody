use crate::core::arena::Arena;
use crate::search::evaluator::Evaluator;
use crate::search::quiescence::quiescence_with_arena;
use bitboard::{
    movegen::{MoveGenerator, generate_legal_moves},
};
use std::io::{self, Write};
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use crate::VERBOSE;

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
pub fn search_node_with_arena<M: MoveGenerator, E: Evaluator>(
    movegen: &M,
    evaluator: &E,
    arena: &mut Arena,
    ply: usize,
    remaining: usize,
    mut alpha: i32,
    beta: i32,
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
