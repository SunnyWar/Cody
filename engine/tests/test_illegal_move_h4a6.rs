use bitboard::movegen::SimpleMoveGen;
use bitboard::movegen::generate_legal_moves;
/// Test to reproduce illegal move recommendation h4a6
///
/// UCI sequence:
/// position startpos moves d2d4 d7d5 b1c3 g8f6 g1f3 e7e6 c1g5 f8e7 g5f6 e7f6
/// e2e4 e8g8 d1d2 c7c5 e4e5 f6e7 d4c5 b8d7 c5c6 b7c6 e1c1 a8b8 d1e1 c6c5 f1d3
/// c5c4 d3f1 d8b6 c3a4 b6a6 a4c3 d7c5 d2e3 c8d7 e3h6 g7h6 e1e3 a6b6 c3d1 g8h8
/// h2h4 c5e4 f1c4 e4f2 e3b3 f2h1 b3b6 b8b6 c4f1 f8g8 c2c3 b6b8 d1e3 h1f2 a2a3
/// d7a4 c3c4 g8c8 b2b4 a7a5
///
/// Engine recommends h4a6 which is illegal (pawn on h4 cannot move to a6)
use bitboard::position::Position;
use engine::search::engine::Engine;
use engine::search::evaluator::MaterialEvaluator;

#[test]
fn test_illegal_move_h4a6_reproduction() {
    println!("\n=== Testing illegal move h4a6 ===");

    // Apply all moves from the UCI sequence
    let moves_str = "d2d4 d7d5 b1c3 g8f6 g1f3 e7e6 c1g5 f8e7 g5f6 e7f6 e2e4 e8g8 d1d2 c7c5 e4e5 \
                     f6e7 d4c5 b8d7 c5c6 b7c6 e1c1 a8b8 d1e1 c6c5 f1d3 c5c4 d3f1 d8b6 c3a4 b6a6 \
                     a4c3 d7c5 d2e3 c8d7 e3h6 g7h6 e1e3 a6b6 c3d1 g8h8 h2h4 c5e4 f1c4 e4f2 e3b3 \
                     f2h1 b3b6 b8b6 c4f1 f8g8 c2c3 b6b8 d1e3 h1f2 a2a3 d7a4 c3c4 g8c8 b2b4 a7a5";
    let moves: Vec<&str> = moves_str.split_whitespace().collect();

    let mut pos = Position::default();
    println!("Starting position: {}", pos.to_fen());

    // Apply all the moves
    for (i, mv_str) in moves.iter().enumerate() {
        println!("\nMove {}: {}", i + 1, mv_str);
        match pos.parse_uci_move(mv_str) {
            Some(mv) => {
                let mut new_pos = pos;
                pos.apply_move_into(&mv, &mut new_pos);
                pos = new_pos;
                println!("  Position after: {}", pos.to_fen());
                println!("  Side to move: {:?}", pos.side_to_move);
            }
            None => {
                panic!("Failed to parse move {} at index {}", mv_str, i);
            }
        }
    }

    println!("\n=== Final position ===");
    println!("FEN: {}", pos.to_fen());
    println!("Side to move: {:?}", pos.side_to_move);

    // Verify that h4a6 is NOT a legal move
    println!("\n=== Checking if h4a6 is legal ===");
    let legal_moves = generate_legal_moves(&pos);
    println!("Number of legal moves: {}", legal_moves.len());

    let h4a6_move = pos.parse_uci_move("h4a6");
    assert!(
        h4a6_move.is_none(),
        "h4a6 should not be a legal move! But parse_uci_move returned: {:?}",
        h4a6_move
    );
    println!("✓ Confirmed: h4a6 is not a legal move");

    // Print all legal moves for debugging
    println!("\n=== Legal moves in this position ===");
    for mv in legal_moves.iter() {
        println!("  {}", mv);
    }

    // Now run the engine search and verify it doesn't return an illegal move
    println!("\n=== Running engine search ===");
    let mut engine = Engine::new(65_536, SimpleMoveGen, MaterialEvaluator);

    let (best_move, score) = engine.search(&pos, 14, None, None);
    println!("Engine best move: {} (score: {})", best_move, score);

    // Verify the best move is legal
    let is_legal = pos.parse_uci_move(&best_move.to_string()).is_some();
    assert!(is_legal, "Engine returned illegal move: {}", best_move);
    println!("✓ Engine returned a legal move");
}
