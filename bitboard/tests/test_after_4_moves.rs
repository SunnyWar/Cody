use bitboard::movegen::generate_legal_moves;
use bitboard::piece::Color;
use bitboard::position::Position;

/// Check if d1b3 is legal after: d2d3, d7d6, c2c3, c7c6
#[test]
fn test_after_4_moves() {
    println!("\n=== Testing position after 4 moves: d2d3, d7d6, c2c3, c7c6 ===\n");

    // After d2d3, d7d6, c2c3, c7c6
    let mut pos = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

    // Apply moves one by one
    let moves = vec!["d2d3", "d7d6", "c2c3", "c7c6"];
    for mv_str in &moves {
        let m = pos
            .parse_uci_move(mv_str)
            .expect(&format!("{} should be legal", mv_str));
        println!("Applying: {}", mv_str);
        let mut new_pos = pos;
        pos.apply_move_into(&m, &mut new_pos);
        pos = new_pos;
    }

    println!("\nFinal position after all 4 moves:");
    println!("FEN: {}", pos.to_fen());
    println!("Side to move: {:?}", pos.side_to_move);
    assert_eq!(
        pos.side_to_move,
        Color::White,
        "After 4 moves, should be White's turn"
    );

    // Generate legal moves
    let legal_moves = generate_legal_moves(&pos);
    println!("Number of legal moves: {}", legal_moves.len());
    println!("First 10 legal moves:");
    for (i, m) in legal_moves.iter().take(10).enumerate() {
        println!("  {}. {}", i + 1, m);
    }

    // Check if d1b3 is legal
    let has_d1b3 = legal_moves.iter().any(|m| {
        m.from().file() == 3 && m.from().rank() == 0 &&  // d1
        m.to().file() == 1 && m.to().rank() == 2 // b3
    });

    println!("\nIs d1b3 in White's legal moves? {}", has_d1b3);

    // Try to parse it
    match pos.parse_uci_move("d1b3") {
        Some(m) => {
            println!("✓ d1b3 parses successfully: {}", m);
            if has_d1b3 {
                println!("✓ Consistent: d1b3 is in both the list and parsed");
            } else {
                println!("!!! INCONSISTENCY: d1b3 parsed but not in legal moves list");
            }
        }
        None => {
            println!("✗ d1b3 failed to parse");
            if has_d1b3 {
                println!("!!! INCONSISTENCY: d1b3 is in list but failed to parse");
            }
        }
    }
}
