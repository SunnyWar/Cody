// bitboard/src/lib.rs

pub mod attack;
pub mod bitboard;
pub mod bitboardmask;
pub mod castling;
pub mod constants;
pub mod mov;
pub mod movegen;
pub mod occupancy;
pub mod perft;
pub mod piece;
pub mod piecebitboards;
pub mod position;
pub mod square;
pub mod tables;
pub mod zobrist;

pub use bitboardmask::BitBoardMask;
pub use perft::perft;
pub use perft::perft_divide;
pub use square::Square;

#[cfg(test)]
mod regression_tests {

    #[test]
    fn test_illegal_move_issue_reproduction() {
        use crate::attack::is_square_attacked;
        use crate::movegen::api::generate_pseudo_moves;
        use crate::piece::Piece;
        use crate::piece::PieceKind;
        use crate::position::Position;

        // Exact position from failing game
        let pos = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let moves = generate_pseudo_moves(&pos);

        // Verify no illegal moves are generated
        // Each generated move should not leave king in check
        for mv in &moves {
            let mut resulting_pos = Position::default();
            pos.apply_move_into(mv, &mut resulting_pos);

            // After the move, find the king square of the side that just moved (now
            // opponent's turn)
            let prev_side = pos.side_to_move;
            let king_piece = Piece::from_parts(prev_side, Some(PieceKind::King));
            let king_bb = resulting_pos.pieces.get(king_piece);

            // Get the king's square - should only be one king
            let king_square = if let Some(sq) = king_bb.squares().next() {
                sq
            } else {
                panic!("King not found for side {:?}", prev_side);
            };

            // Check if king is attacked by opponent
            let board_state = resulting_pos.to_board_state();
            let is_attacked = is_square_attacked(king_square, prev_side.opposite(), &board_state);

            assert!(
                !is_attacked,
                "Illegal move generated: king left in check after {:?}",
                mv
            );
        }

        // Also verify that at least SOME legal moves exist
        assert!(!moves.is_empty(), "No moves generated from position");
    }
}

#[cfg(test)]
mod regression_tests {

#[test]
fn test_evaluation_differences() {
    use crate::position::Position;

    // Balanced starting position
    let balanced = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    let balanced_eval = evaluate(&balanced);

    // White up a rook - should be significantly better for white
    let white_up_rook = Position::from_fen("k7/8/8/8/8/8/8/K6R w - - 0 1").unwrap();
    let white_eval = evaluate(&white_up_rook);

    // Difference should reflect rook value (roughly 500cp)
    let diff = white_eval - balanced_eval;
    assert!(diff > 300, "Rook advantage not reflected in eval. Diff: {}", diff);
}

}
