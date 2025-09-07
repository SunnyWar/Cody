#![allow(non_snake_case)]

use cody::core::position::Position;
use cody::search::engine::NODE_COUNT;
use cody::search::movegen::MoveGenerator;
use cody::search::{engine::Engine, evaluator::MaterialEvaluator, movegen::SimpleMoveGen};
use std::sync::atomic::Ordering;

fn main() {
    let mut engine = Engine::new(65536, SimpleMoveGen, MaterialEvaluator);
    let pos = Position::default();

    println!("Starting position:");
    println!("FEN: {}", pos.to_fen());

    let moves = SimpleMoveGen.generate_moves(&pos);
    println!("Generated {} moves", moves.len());
    for m in &moves {
        println!("{:?}", m);
    }

    let depth = 4;
    let score = engine.search(&pos, depth);

    println!(
        "info depth {} nodes {}",
        depth,
        NODE_COUNT.load(Ordering::Relaxed)
    );
    println!("Search result: {}", score);
}
