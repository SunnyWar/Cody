use bitboard::movegen::generate_legal_moves;
use bitboard::piece::Color;
use bitboard::position::Position;

/// Reproduce the exact scenario from the bug report
#[test]
fn test_exact_bug_scenario() {
    println!("\n=== Reproducing exact bug scenario ===\n");

    // After moves: d2d3, d7d6, c2c3
    // The position should be Black to move
    let pos_after_3_moves =
        Position::from_fen("rnbqkbnr/ppp1pppp/3p4/8/8/2PP4/PP2PPPP/RNBQKBNR b KQkq - 0 2");

    println!("Position after d2d3, d7d6, c2c3:");
    println!("FEN: {}", pos_after_3_moves.to_fen());
    println!("Side to move: {:?}", pos_after_3_moves.side_to_move);
    assert_eq!(
        pos_after_3_moves.side_to_move,
        Color::Black,
        "Should be Black's turn"
    );

    // Generate legal moves for this position
    let legal_moves = generate_legal_moves(&pos_after_3_moves);
    println!("Number of legal moves: {}", legal_moves.len());
    println!("First 10 legal moves:");
    for (i, m) in legal_moves.iter().take(10).enumerate() {
        println!("  {}. {}", i + 1, m);
    }

    // The bug report says engine returned d1b3 when it was Black's turn
    // Check if d1b3 is in the legal move list
    let has_d1b3 = legal_moves.iter().any(|m| {
        m.from().file() == 3 && m.from().rank() == 0 &&  // d1
        m.to().file() == 1 && m.to().rank() == 2 // b3
    });

    println!(
        "\nIs d1b3 (White Queen move) in Black's legal moves? {}",
        has_d1b3
    );
    assert!(!has_d1b3, "d1b3 should NOT be legal for Black");

    // Also verify parsing fails
    match pos_after_3_moves.parse_uci_move("d1b3") {
        Some(m) => panic!(
            "ERROR: d1b3 parsed as {:?} but should fail when it's Black's turn",
            m
        ),
        None => println!("✓ Correctly: parse_uci_move(\"d1b3\") returns None when Black to move"),
    }

    // Verify Black CAN move c7c6
    match pos_after_3_moves.parse_uci_move("c7c6") {
        Some(m) => println!("✓ Correctly: parse_uci_move(\"c7c6\") succeeds: {}", m),
        None => panic!("ERROR: c7c6 should be legal for Black"),
    }
}
