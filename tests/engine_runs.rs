// tests/engine_runs.rs
use cody::{Engine, MaterialEvaluator, SimpleMoveGen, TEST_CASES};

macro_rules! gen_tests {
    ($($name:ident: $idx:expr),* $(,)?) => {
        $(
            #[test]
            fn $name() {
                let case = &TEST_CASES[$idx];
                let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
                let (_, score) = engine.search(&case.position(), 4);
                assert_eq!(
                    score,
                    case.expected_score,
                    "Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
                    case.name,
                    case.fen,
                    case.expected_score,
                    score
                );

                // later: assert_eq!(best_move, case.expected_move);
            }
        )*
    }
}

#[cfg(not(rust_analyzer))]
include!(concat!(env!("OUT_DIR"), "/generated_tests.rs"));
