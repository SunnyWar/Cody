// src/search/engine.rs

use crate::core::arena::Arena;
use crate::search::evaluator::Evaluator;
use bitboard::{
    mov::ChessMove,
    movegen::{MoveGenerator, generate_legal_moves, generate_pseudo_moves},
    position::Position,
};
use rand::prelude::IndexedRandom;
use std::sync::atomic::{AtomicU64, Ordering};
pub static NODE_COUNT: AtomicU64 = AtomicU64::new(0);

const MATE_SCORE: i32 = -1;

pub struct Engine<M: MoveGenerator, E: Evaluator> {
    arena: Arena,
    movegen: M,
    evaluator: E,
}

impl<M: MoveGenerator, E: Evaluator> Engine<M, E> {
    pub fn new(arena_size: usize, movegen: M, evaluator: E) -> Self {
        Self {
            arena: Arena::new(arena_size),
            movegen,
            evaluator,
        }
    }

    pub fn search(&mut self, root: &Position, depth: usize) -> (ChessMove, i32) {
        self.arena.reset();
        self.arena.get_mut(0).position.copy_from(root);

        let start_time = std::time::Instant::now();

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

        for m in moves {
            {
                let (parent, child) = self.arena.get_pair_mut(0, 1);
                parent.position.apply_move_into(&m, &mut child.position);
            }
            let score = -self.search_node(1, depth - 1);

            if score > best_score {
                best_score = score;
                best_move = m;
            }
        }

        (best_move, best_score)
    }

    fn search_node(&mut self, ply: usize, remaining: usize) -> i32 {
        NODE_COUNT.fetch_add(1, Ordering::Relaxed);

        if remaining == 0 {
            return self.evaluator.evaluate(&self.arena.get(ply).position);
        }

        let moves = {
            let (parent, _) = self.arena.get_pair_mut(ply, ply + 1);
            generate_legal_moves(&parent.position)
        };

        if moves.is_empty() {
            let pos = &self.arena.get(ply).position;
            if self.movegen.in_check(pos) {
                // Checkmate: losing sooner is worse
                return -MATE_SCORE + ply as i32;
            }
            return 0;
        }

        let mut best_score = i32::MIN;
        for m in moves {
            {
                let (parent, child) = self.arena.get_pair_mut(ply, ply + 1);
                parent.position.apply_move_into(&m, &mut child.position);
            }
            let score = -self.search_node(ply + 1, remaining - 1);
            best_score = best_score.max(score);
        }

        best_score
    }

    pub fn clear_state(&self) {
        NODE_COUNT.store(0, Ordering::Relaxed)
    }
}
