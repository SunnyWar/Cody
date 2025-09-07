use crate::core::{arena::Arena, position::Position};
use crate::search::evaluator::Evaluator;
use crate::search::movegen::MoveGenerator;

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
        self.arena.get_mut(0).position.copy_from(root);
        self.search_node(0, depth)
    }

    fn search_node(&mut self, ply: usize, remaining: usize) -> i32 {
        NODE_COUNT.fetch_add(1, Ordering::Relaxed);

        if remaining == 0 {
            return self.evaluator.evaluate(&self.arena.get(ply).position);
        }

        let moves;
        {
            let (parent, child) = self.arena.get_pair_mut(ply, ply + 1);
            moves = self.movegen.generate_moves(&parent.position);

            for m in &moves {
                parent.position.apply_move_into(m, &mut child.position);
            }
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
}
