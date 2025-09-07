use crate::core::position::{Move, Position};
use crate::search::traits::MoveGenerator;

pub struct SimpleMoveGen;

impl MoveGenerator for SimpleMoveGen {
    fn generate_moves(&self, _pos: &Position) -> Vec<Move> {
        // Phase 1: stub
        Vec::new()
    }
}
