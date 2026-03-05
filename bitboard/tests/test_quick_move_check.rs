use bitboard::movegen::generate_legal_moves;
use bitboard::piece::Color;
/// Quick test to check if move generation is correct
use bitboard::position::Position;
use bitboard::square::Square;

#[test]
fn test_no_d1b3_when_black_to_move() {
    // After d2d3, d7d6, c2c3, it's Black's turn
    let pos = Position::from_fen("rnbqkbnr/ppp1pppp/3p4/8/8/2PP4/PP2PPPP/RNBQKBNR b KQkq - 0 2");

    assert_eq!(pos.side_to_move, Color::Black);

    // Generate legal moves
    let moves = generate_legal_moves(&pos);

    // Check d1b3 is not in the list
    let d1b3_present = moves
        .iter()
        .any(|m| m.from() == Square::D1 && m.to() == Square::B3);

    assert!(
        !d1b3_present,
        "d1b3 should not be a legal move when Black is to move"
    );
    println!("✓ Test passed: d1b3 not in legal moves");
}
