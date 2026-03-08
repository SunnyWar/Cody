// bitboard/src/lib.rs

pub mod attack;
pub mod bitboard;
pub mod bitboardmask;
pub mod castling;
pub mod constants;
pub mod intrinsics;
pub mod mov;
pub mod movegen;
pub mod movelist;
pub mod occupancy;
pub mod perft;
pub mod piece;
pub mod piecebitboards;
pub mod position;
pub mod square;
pub mod tables;
pub mod zobrist;

pub use bitboardmask::BitBoardMask;
pub use movelist::MoveList;
pub use perft::perft;
pub use perft::perft_divide;
pub use square::Square;

#[cfg(test)]
mod regression_tests {

    #[test]
    fn test_illegal_move_issue_reproduction() {
        use crate::attack::is_square_attacked;
        use crate::movegen::api::generate_pseudo_moves_fast;
        use crate::piece::Piece;
        use crate::piece::PieceKind;
        use crate::position::Position;

        // Exact position from failing game
        let pos = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let moves = generate_pseudo_moves_fast(&pos);

        // Verify no illegal moves are generated
        // Each generated move should not leave king in check
        for mv in moves.iter() {
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
