use crate::core::position::{Position};
use crate::core::mov::{Move};

pub trait MoveGenerator {
    fn generate_moves(&self, pos: &Position) -> Vec<Move>;
}

pub trait Evaluator {
    fn evaluate(&self, pos: &Position) -> i32;
}
