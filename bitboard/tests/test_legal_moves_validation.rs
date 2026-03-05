use bitboard::movegen::generate_legal_moves;
use bitboard::piece::Color;
/// Test to check if generate_legal_moves returns moves for the correct side
use bitboard::position::Position;

#[test]
fn test_legal_moves_for_black_turn() {
    println!("\n=== Testing legal moves when it's Black's turn ===");

    // After d2d3, d7d6, c2c3, it should be Black's turn
    let pos = Position::from_fen("rnbqkbnr/ppp1pppp/3p4/8/8/2PP4/PP2PPPP/RNBQKBNR b KQkq - 0 2");

    println!("Position: {}", pos.to_fen());
    println!("Side to move: {:?}", pos.side_to_move);
    assert_eq!(pos.side_to_move, Color::Black, "Should be Black's turn");

    // Generate legal moves
    let moves = generate_legal_moves(&pos);
    println!("Number of legal moves: {}", moves.len());

    // Check that no moves start from d1 (White queen position)
    let white_queen_moves = moves.iter().filter(|m| {
        let from = m.from();
        println!("  Move: {} (from: {:?})", m, from);
        from.file() == 3 && from.rank() == 0 // d1
    });

    let white_queen_vec: Vec<_> = white_queen_moves.collect();

    if !white_queen_vec.is_empty() {
        println!("\n!!! BUG DETECTED !!!");
        println!(
            "Found {} moves starting from d1 (White Queen):",
            white_queen_vec.len()
        );
        for m in white_queen_vec {
            println!("  - {}", m);
        }
        panic!("generate_legal_moves should not generate White Queen moves when it's Black's turn");
    } else {
        println!("\n✓ Correctly: No moves from d1 (White Queen) when it's Black's turn");
    }

    // Verify all moves are from Black pieces
    println!("\nVerifying all moves are from Black pieces:");
    for m in moves.iter().take(10) {
        let from = m.from();
        let piece = pos.pieces.find_piece(from);
        match piece {
            Some(p) => {
                println!("  Move {} from {:?} - Piece: {:?}", m, from, p);
                assert_eq!(p.color(), Color::Black, "Move should be from Black piece");
            }
            None => {
                println!("  WARNING: Move {} from empty square {:?}", m, from);
                panic!("Move starts from empty square");
            }
        }
    }
}

#[test]
fn test_specific_d1b3_move() {
    println!("\n=== Testing if d1b3 is legal when it's Black's turn ===");

    let pos = Position::from_fen("rnbqkbnr/ppp1pppp/3p4/8/8/2PP4/PP2PPPP/RNBQKBNR b KQkq - 0 2");
    println!("Position: {}", pos.to_fen());
    println!("Side to move: {:?}", pos.side_to_move);

    // Try to parse d1b3
    match pos.parse_uci_move("d1b3") {
        Some(m) => {
            println!("\n!!! BUG: d1b3 was parsed as legal !!!");
            println!("Move: {}", m);
            panic!("d1b3 should not be legal when it's Black's turn");
        }
        None => {
            println!("✓ d1b3 correctly failed to parse");
        }
    }

    // Generate all legal moves and check if d1b3 is in the list
    let moves = generate_legal_moves(&pos);
    let d1b3_in_list = moves.iter().any(|m| {
        m.from().file() == 3 && m.from().rank() == 0 && m.to().file() == 1 && m.to().rank() == 2
    });

    if d1b3_in_list {
        println!("!!! BUG: d1b3 found in legal moves list when it's Black's turn!");
        panic!("generate_legal_moves returned illegal White Queen move");
    } else {
        println!("✓ d1b3 is not in legal moves list");
    }
}
