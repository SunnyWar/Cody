#[cfg(test)]
mod tests {
    use crate::core::position::Position;
    use crate::search::{engine::Engine, evaluator::MaterialEvaluator, movegen::SimpleMoveGen};

    #[test]
    fn test_engine_runs() {
        let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
        let pos = Position::default();
        let score = engine.search(&pos, 1);
        assert_eq!(score, 0);
    }
}
