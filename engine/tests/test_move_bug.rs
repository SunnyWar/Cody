use bitboard::piece::Color;
/// Test to reproduce the UCI move parsing bug where engine suggests illegal
/// moves This reproduces the sequence: d2d3, d7d6, c2c3, then d1b3
/// where d1b3 is illegal because it's Black's turn to move.
use bitboard::position::Position;

#[test]
fn test_move_sequence_bug_reproduction() {
    println!("\n=== Testing move sequence: d2d3, d7d6, c2c3, d1b3 ===");

    // Start from the beginning
    let pos0 = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    println!("Starting position: {}", pos0.to_fen());

    // Move 1: d2d3 (White pawn)
    let mv1 = pos0.parse_uci_move("d2d3").expect("d2d3 should be valid");
    println!("Move 1: d2d3");
    let mut pos1 = pos0;
    pos0.apply_move_into(&mv1, &mut pos1);
    println!("  Position after: {}", pos1.to_fen());
    println!("  Side to move: {:?}", pos1.side_to_move);
    assert_eq!(
        pos1.side_to_move,
        Color::Black,
        "After d2d3, should be Black's turn"
    );

    // Move 2: d7d6 (Black pawn)
    let mv2 = pos1.parse_uci_move("d7d6").expect("d7d6 should be valid");
    println!("\nMove 2: d7d6");
    let mut pos2 = pos1;
    pos1.apply_move_into(&mv2, &mut pos2);
    println!("  Position after: {}", pos2.to_fen());
    println!("  Side to move: {:?}", pos2.side_to_move);
    assert_eq!(
        pos2.side_to_move,
        Color::White,
        "After d7d6, should be White's turn"
    );

    // Move 3: c2c3 (White pawn)
    let mv3 = pos2.parse_uci_move("c2c3").expect("c2c3 should be valid");
    println!("\nMove 3: c2c3");
    let mut pos3 = pos2;
    pos2.apply_move_into(&mv3, &mut pos3);
    println!("  Position after: {}", pos3.to_fen());
    println!("  Side to move: {:?}", pos3.side_to_move);
    assert_eq!(
        pos3.side_to_move,
        Color::Black,
        "After c2c3, should be Black's turn"
    );

    // Verify the current position is correct
    println!("\n=== Current position (after 3 moves) ===");
    println!("FEN: {}", pos3.to_fen());
    println!("Side to move: {:?} (should be Black)", pos3.side_to_move);
    assert_eq!(
        pos3.side_to_move,
        Color::Black,
        "After 3 moves, it should be Black's turn"
    );

    // Move 4: Try to parse d1b3 (White queen move - ILLEGAL because it's Black's
    // turn)
    println!("\n=== Attempting illegal move 4: d1b3 ===");
    println!("Current position: {}", pos3.to_fen());
    println!("Current side to move: {:?}", pos3.side_to_move);
    println!("Move attempted: d1b3 (White Queen from d1 to b3)");

    // This attempts to parse a move for the WRONG side
    match pos3.parse_uci_move("d1b3") {
        Some(mv4) => {
            // If parsing succeeds, that's the bug - we're parsing a White move when it's
            // Black's turn
            println!("\n!!! BUG DETECTED !!!");
            println!("Move parsed successfully: {:?}", mv4);
            println!("But it's Black's turn! This is an illegal move.");
            println!("The parser should not generate moves for the opponent side.");
            panic!("d1b3 should not parse as a legal move when it's Black's turn to move");
        }
        None => {
            println!("\n✓ Move parsing correctly failed");
            println!("This is expected - d1b3 is not a legal move for Black.");
        }
    }
}

#[test]
fn test_move_sequence_with_correct_black_move() {
    println!("\n=== Testing correct sequence: d2d3, d7d6, c2c3, c7c6 ===");

    // Start from the beginning
    let pos0 = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

    // Move 1: d2d3
    let mv1 = pos0.parse_uci_move("d2d3").expect("d2d3 should be valid");
    let mut pos1 = pos0;
    pos0.apply_move_into(&mv1, &mut pos1);
    println!("Move 1: d2d3 - OK");

    // Move 2: d7d6
    let mv2 = pos1.parse_uci_move("d7d6").expect("d7d6 should be valid");
    let mut pos2 = pos1;
    pos1.apply_move_into(&mv2, &mut pos2);
    println!("Move 2: d7d6 - OK");

    // Move 3: c2c3
    let mv3 = pos2.parse_uci_move("c2c3").expect("c2c3 should be valid");
    let mut pos3 = pos2;
    pos2.apply_move_into(&mv3, &mut pos3);
    println!("Move 3: c2c3 - OK");
    assert_eq!(pos3.side_to_move, Color::Black);

    // Move 4: c7c6 (correct Black move)
    let mv4 = pos3.parse_uci_move("c7c6").expect("c7c6 should be valid");
    let mut pos4 = pos3;
    pos3.apply_move_into(&mv4, &mut pos4);
    println!("Move 4: c7c6 - OK");
    assert_eq!(pos4.side_to_move, Color::White);

    // Now it should be White's turn
    println!("\nAfter correct Black move c7c6:");
    println!("Position: {}", pos4.to_fen());
    println!("Side to move: {:?} (should be White)", pos4.side_to_move);

    // Move 5: Try d1b3 (now it should be legal)
    let mv5 = pos4
        .parse_uci_move("d1b3")
        .expect("d1b3 should be valid now");
    let mut pos5 = pos4;
    pos4.apply_move_into(&mv5, &mut pos5);
    println!("Move 5: d1b3 - OK");
    println!("✓ Sequence completed successfully with correct side-to-move logic");
}
