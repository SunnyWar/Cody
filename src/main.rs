use Cody::core::position::Position;
use Cody::search::{engine::Engine, evaluator::MaterialEvaluator, movegen::SimpleMoveGen};

fn main() {
    let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
    let pos = Position::default();
    let score = engine.search(&pos, 1);
    println!("Search result: {}", score);
}
