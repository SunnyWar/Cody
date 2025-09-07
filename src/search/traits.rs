use crate::core::mov::Move;
use crate::core::position::Position;

pub trait MoveGenerator {
    fn generate_moves(&self, pos: &Position) -> Vec<Move>;
}

pub trait Evaluator {
    fn evaluate(&self, pos: &Position) -> i32;
}
