// Test make_move and unmake_move functionality

use bitboard::movegen::generate_legal_moves;
use bitboard::position::Position;

#[test]
fn test_make_unmake_preserves_position() {
    // Test that make_move followed by unmake_move restores original position
    let original = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

    // Generate all legal moves
    let moves = generate_legal_moves(&original);

    // Test each move
    for mv in moves.iter() {
        let mut pos = original;
        let undo = pos.make_move(mv);

        // Verify position changed
        assert_ne!(pos.to_fen(), original.to_fen());

        // Unmake the move
        pos.unmake_move(mv, &undo);

        // Verify position is restored
        assert_eq!(pos.to_fen(), original.to_fen());
    }
}

#[test]
fn test_make_unmake_complex_position() {
    // Test with a more complex position including castling rights and EP
    let original =
        Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");

    let moves = generate_legal_moves(&original);

    for mv in moves.iter() {
        let mut pos = original;
        let undo = pos.make_move(mv);
        pos.unmake_move(mv, &undo);

        // Check all position fields are restored
        assert_eq!(pos.to_fen(), original.to_fen());
        assert_eq!(pos.ep_square, original.ep_square);
        assert_eq!(pos.halfmove_clock, original.halfmove_clock);
        assert_eq!(pos.fullmove_number, original.fullmove_number);
    }
}

#[test]
fn test_make_unmake_capture() {
    // Test capturing moves specifically
    let original =
        Position::from_fen("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2");

    let moves = generate_legal_moves(&original);

    // Find a capturing move (e4xd5)
    let capture_move = moves.iter().find(|m| m.to.to_string() == "d5").unwrap();

    let mut pos = original;
    let undo = pos.make_move(capture_move);

    // Verify capture happened
    assert_ne!(pos.to_fen(), original.to_fen());

    // Unmake
    pos.unmake_move(capture_move, &undo);

    // Verify restored
    assert_eq!(pos.to_fen(), original.to_fen());
}

#[test]
fn test_make_unmake_castling() {
    // Test castling moves
    let original = Position::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1");

    let moves = generate_legal_moves(&original);

    for mv in moves.iter() {
        let mut pos = original;
        let undo = pos.make_move(mv);
        pos.unmake_move(mv, &undo);
        assert_eq!(pos.to_fen(), original.to_fen());
    }
}

#[test]
fn test_make_unmake_en_passant() {
    // Test en passant capture
    let original =
        Position::from_fen("rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3");

    let moves = generate_legal_moves(&original);

    // Find en passant move (e5xf6)
    let ep_move = moves
        .iter()
        .find(|m| m.to.to_string() == "f6" && m.from.to_string() == "e5");

    if let Some(ep_move) = ep_move {
        let mut pos = original;
        let undo = pos.make_move(ep_move);
        pos.unmake_move(ep_move, &undo);
        assert_eq!(pos.to_fen(), original.to_fen());
    }
}

#[test]
fn test_make_unmake_promotion() {
    // Test pawn promotion
    let original = Position::from_fen("8/P7/8/8/8/8/8/k6K w - - 0 1");

    let moves = generate_legal_moves(&original);

    for mv in moves.iter() {
        let mut pos = original;
        let undo = pos.make_move(mv);
        pos.unmake_move(mv, &undo);
        assert_eq!(pos.to_fen(), original.to_fen());
    }
}
