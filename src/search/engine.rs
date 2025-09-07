use crate::core::{arena::Arena, position::Position};
use crate::search::traits::{Evaluator, MoveGenerator};

use std::sync::atomic::{AtomicU64, Ordering};
pub static NODE_COUNT: AtomicU64 = AtomicU64::new(0);

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

    pub fn search(&mut self, root: &Position, depth: usize) -> i32 {
        NODE_COUNT.store(0, Ordering::Relaxed);
        self.arena.reset();
        let root_idx = self.arena.alloc().expect("Arena full");
        self.arena.get_mut(root_idx).position = root.clone();
        self.search_node(root_idx, depth)
    }

    fn search_node(&mut self, node_idx: usize, depth: usize) -> i32 {
        NODE_COUNT.fetch_add(1, Ordering::Relaxed);
        
        if depth == 0 {
            let score = self.evaluator.evaluate(&self.arena.get(node_idx).position);
            self.arena.get_mut(node_idx).score = score;
            return score;
        }

        let moves = self
            .movegen
            .generate_moves(&self.arena.get(node_idx).position);
        let mut best_score = i32::MIN;

        for mv in moves {
            if let Some(child_idx) = self.arena.alloc() {
                let new_pos = self.arena.get(node_idx).position.apply_move(&mv);
                self.arena.get_mut(child_idx).position = new_pos;
                let score = self.search_node(child_idx, depth - 1);
                best_score = best_score.max(score);
            }
        }

        self.arena.get_mut(node_idx).score = best_score;
        best_score
    }
}
