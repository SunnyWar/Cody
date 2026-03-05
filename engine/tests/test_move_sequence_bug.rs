use bitboard::piece::Color;
/// Test to reproduce the UCI move parsing bug where engine suggests illegal
/// moves This reproduces the sequence: d2d3, d7d6, c2c3, d1b3
/// where d1b3 is illegal because it's Black's turn to move.
use bitboard::position::Position;

#[test]
fn test_move_sequence_bug_reproduction() {
    // Start from the beginning
    let pos = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

    // Move 1: d2d3 (White pawn)
    let mv1 = pos.parse_uci_move("d2d3").expect("d2d3 should be valid");
    println!("Move 1: d2d3");
    println!("  FEN before: {}", pos.to_fen());
    let mut pos2 = pos;
    pos.apply_move_into(&mv1, &mut pos2);
    println!("  FEN after: {}", pos2.to_fen());
    println!(
        "  To move: {}",
        if pos2.side_to_move == Color::White {
            "White"
        } else {
            "Black"
        }
    );

    // Move 2: d7d6 (Black pawn)
    let mv2 = pos2.parse_uci_move("d7d6").expect("d7d6 should be valid");
    println!("\nMove 2: d7d6");
    println!("  FEN before: {}", pos2.to_fen());
    let mut pos3 = pos2;
    pos2.apply_move_into(&mv2, &mut pos3);
    println!("  FEN after: {}", pos3.to_fen());
    println!(
        "  To move: {}",
        if pos3.side_to_move == Color::White {
            "White"
        } else {
            "Black"
        }
    );

    // Move 3: c2c3 (White pawn)
    let mv3 = pos3.parse_uci_move("c2c3").expect("c2c3 should be valid");
    println!("\nMove 3: c2c3");
    println!("  FEN before: {}", pos3.to_fen());
    let mut pos4 = pos3;
    pos3.apply_move_into(&mv3, &mut pos4);
    println!("  FEN after: {}", pos4.to_fen());
    println!(
        "  To move: {}",
        if pos4.side_to_move == Color::White {
            "White"
        } else {
            "Black"
        }
    );

    // After 3 moves (odd number), it should be Black's turn
    assert!(
        pos4.side_to_move == Color::Black,
        "After 3 moves, it should be Black's turn"
    );

    // Move 4: Try to parse d1b3 (White queen move - ILLEGAL because it's Black's
    // turn)
    println!("\nAttempting move 4: d1b3");
    println!("  FEN before: {}", pos4.to_fen());
    println!("  Is White to move: {}", pos4.side_to_move == Color::White);

    // This should either fail gracefully or the move should be invalid
    match pos4.parse_uci_move("d1b3") {
        Some(mv4) => {
            // If parsing succeeds, the move itself might still be illegal
            println!("  Move parsed successfully: {:?}", mv4);
            println!("  WARNING: d1b3 was parsed but it's Black's turn!");
            assert!(
                false,
                "d1b3 should not be a legal move when it's Black's turn"
            );
        }
        None => {
            println!("  Move parsing failed (expected)");
            println!("  This correctly identifies the illegal move");
        }
    }
}

#[test]
fn test_move_sequence_with_correct_black_move() {
    // Test the sequence with the correct Black move
    let pos = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

    // Move 1: d2d3
    let mv1 = pos.parse_uci_move("d2d3").expect("d2d3 should be valid");
    let mut pos2 = pos;
    pos.apply_move_into(&mv1, &mut pos2);

    // Move 2: d7d6
    let mv2 = pos2.parse_uci_move("d7d6").expect("d7d6 should be valid");
    let mut pos3 = pos2;
    pos2.apply_move_into(&mv2, &mut pos3);

    // Move 3: c2c3
    let mv3 = pos3.parse_uci_move("c2c3").expect("c2c3 should be valid");
    let mut pos4 = pos3;
    pos3.apply_move_into(&mv3, &mut pos4);

    // Verify it's Black's turn
    assert!(pos4.side_to_move == Color::Black, "Should be Black's turn");
    println!("Position after 3 moves: {}", pos4.to_fen());

    // Move 4: c7c6 (correct Black move)
    let mv4 = pos4.parse_uci_move("c7c6").expect("c7c6 should be valid");
    println!("Move 4: c7c6");
    println!("  FEN before: {}", pos4.to_fen());
    let mut pos5 = pos4;
    pos4.apply_move_into(&mv4, &mut pos5);
    println!("  FEN after: {}", pos5.to_fen());

    // Now it should be White's turn
    assert!(
        pos5.side_to_move == Color::White,
        "After 4 moves, it should be White's turn"
    );

    // Move 5: Try d1b3 (now it should be legal)
    let mv5 = pos5
        .parse_uci_move("d1b3")
        .expect("d1b3 should be valid now");
    println!("Move 5: d1b3");
    println!("  FEN before: {}", pos5.to_fen());
    let mut pos6 = pos5;
    pos5.apply_move_into(&mv5, &mut pos6);
    println!("  FEN after: {}", pos6.to_fen());
}
