use crate::api::uciapi::CodyApi;
use bitboard::Square;

#[test]
fn test_uci_position_moves_c3d5_state_consistency() {
    let api = &mut CodyApi::new();
    let mut out = std::io::sink();
    // Simulate: position fen ... moves c3d5
    let fen = "r2q1rk1/1p2bppp/p1npbn2/4p3/P3P3/1NN5/1PP1BPPP/R1BQ1R1K w - - 0 1";

    // Print all pseudo-legal moves for the FEN
    let pos = bitboard::position::Position::from_fen(fen);
    let mv = pos.parse_uci_move("c3d5").expect("Should parse c3d5");
    let mut new_pos = bitboard::position::Position::default();
    pos.apply_move_into(&mv, &mut new_pos);
    assert_eq!(
        new_pos.side_to_move,
        bitboard::piece::Color::Black,
        "Direct apply_move_into: should be Black to move after c3d5"
    );
    eprintln!("FEN after direct apply_move_into: {}", new_pos.to_fen());

    let cmd = format!("position fen {} moves c3d5", fen);
    api.handle_position(&cmd, &mut out);

    // After parsing, engine should think it is Black's turn
    eprintln!("FEN after UCI handler: {}", api.current_pos.to_fen());
    assert_eq!(
        api.current_pos.side_to_move,
        bitboard::piece::Color::Black,
        "Engine should have Black to move after c3d5"
    );

    // Helper to get piece at a square
    fn get_piece_at(
        pos: &bitboard::position::Position,
        sq: Square,
    ) -> Option<bitboard::piece::Piece> {
        let mask = bitboard::BitBoardMask::from_square(sq);
        for (piece, bb) in pos.pieces.iter() {
            if (bb & mask).is_nonempty() {
                return Some(piece);
            }
        }
        None
    }

    // C3 should be empty
    assert!(
        get_piece_at(&api.current_pos, Square::C3).is_none(),
        "C3 should be empty after c3d5"
    );

    // Black should have no legal moves from C3
    let black_moves = bitboard::movegen::generate_legal_moves(&api.current_pos);
    assert!(
        black_moves.iter().all(|m| m.from() != Square::C3),
        "No legal move should originate from C3 for Black"
    );

    // (Optional) Check if engine's best move is not from C3 (if search/TT is
    // accessible) let (bm, _score) = api.engine.search(&api.current_pos, 1,
    // Some(10), None); assert!(bm.from() != Square::C3, "Engine search
    // should not return a move from C3");
}
