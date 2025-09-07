#![allow(non_snake_case)]

use cody::core::position::Position;
use cody::search::engine::NODE_COUNT;
use cody::search::movegen::MoveGenerator;
use cody::search::{engine::Engine, evaluator::MaterialEvaluator, movegen::SimpleMoveGen};
use std::sync::atomic::Ordering;

fn main() {
    let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
    let pos = Position::default();

    println!("Starting position:");
    println!("FEN: {}", pos.to_fen());

    let moves = SimpleMoveGen.generate_moves(&pos);
    println!("Generated {} moves", moves.len());
    for m in &moves {
        println!("{:?}", m);
    }

    NODE_COUNT.store(0, Ordering::Relaxed);
    let score = engine.search(&pos, 1);
    println!("info nodes {}", NODE_COUNT.load(Ordering::Relaxed));
    println!("Search result: {}", score);
}
