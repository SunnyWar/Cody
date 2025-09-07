use crate::core::position::{Move, Position};

pub trait MoveGenerator {
    fn generate_moves(&self, pos: &Position) -> Vec<Move>;
}

pub trait Evaluator {
    fn evaluate(&self, pos: &Position) -> i32;
}
