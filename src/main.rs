#![allow(non_snake_case)]

use cody::core::position::Position;
use cody::search::{engine::Engine, evaluator::MaterialEvaluator, movegen::SimpleMoveGen};

fn main() {
    let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
    let pos = Position::default();
    let score = engine.search(&pos, 1);
    println!("Search result: {}", score);
}
