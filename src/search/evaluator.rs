use crate::core::position::Position;
use crate::search::traits::Evaluator;

pub struct MaterialEvaluator;

impl Evaluator for MaterialEvaluator {
    fn evaluate(&self, _pos: &Position) -> i32 {
        // Phase 1: stub: material balance = 0
        0
    }
}
