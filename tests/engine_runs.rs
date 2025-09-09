// tests/engine_runs.rs

use cody::{Engine, MaterialEvaluator, SimpleMoveGen, TEST_POSITIONS};

#[test]
fn engine_runs_on_all_positions() {
    let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
    for pos in TEST_POSITIONS.iter() {
        let _score = engine.search(pos, 1);
    }
}
