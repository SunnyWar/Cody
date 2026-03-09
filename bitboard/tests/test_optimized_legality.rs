use bitboard::movegen::generate_pseudo_moves_fast;
use bitboard::movegen::is_move_legal_without_making;
use bitboard::position::Position;

#[test]
fn test_optimized_legality_check_matches_make_unmake() {
    let test_positions = vec![
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
    ];

    for fen in test_positions {
        let pos = Position::from_fen(fen);
        let pseudo_moves = generate_pseudo_moves_fast(&pos);

        for mv in pseudo_moves.as_slice() {
            // Check using old method (make/unmake)
            let mut test_pos = pos;
            let undo = test_pos.make_move(mv);
            let mut legal_test = test_pos;
            legal_test.side_to_move = pos.side_to_move;
            let old_result = !bitboard::movegen::is_in_check(&legal_test, pos.side_to_move);
            test_pos.unmake_move(mv, &undo);

            // Check using new optimized method
            let new_result = is_move_legal_without_making(&pos, mv);

            assert_eq!(
                old_result, new_result,
                "Legality mismatch for move {:?} in position {}",
                mv, fen
            );
        }
    }
}

#[test]
fn test_optimized_legality_check_castling() {
    let pos = Position::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1");
    let pseudo_moves = generate_pseudo_moves_fast(&pos);

    for mv in pseudo_moves.as_slice() {
        let old_result = {
            let mut test_pos = pos;
            let undo = test_pos.make_move(mv);
            let mut legal_test = test_pos;
            legal_test.side_to_move = pos.side_to_move;
            let result = !bitboard::movegen::is_in_check(&legal_test, pos.side_to_move);
            test_pos.unmake_move(mv, &undo);
            result
        };

        let new_result = is_move_legal_without_making(&pos, mv);

        assert_eq!(
            old_result, new_result,
            "Castling legality mismatch for move {:?}",
            mv
        );
    }
}

#[test]
fn test_optimized_legality_check_en_passant() {
    let pos = Position::from_fen("rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3");
    let pseudo_moves = generate_pseudo_moves_fast(&pos);

    for mv in pseudo_moves.as_slice() {
        let old_result = {
            let mut test_pos = pos;
            let undo = test_pos.make_move(mv);
            let mut legal_test = test_pos;
            legal_test.side_to_move = pos.side_to_move;
            let result = !bitboard::movegen::is_in_check(&legal_test, pos.side_to_move);
            test_pos.unmake_move(mv, &undo);
            result
        };

        let new_result = is_move_legal_without_making(&pos, mv);

        assert_eq!(
            old_result, new_result,
            "En passant legality mismatch for move {:?}",
            mv
        );
    }
}

#[test]
fn test_optimized_legality_check_pins() {
    let test_positions = vec![
        "rnbqk2r/pppp1ppp/5n2/2b1p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4",
        "r1bqkbnr/pppp1ppp/2n5/4p3/3PP3/8/PPP2PPP/RNBQKBNR w KQkq - 0 3",
        "8/8/4k3/3pP3/8/8/3K4/8 w - d6 0 1",
    ];

    for fen in test_positions {
        let pos = Position::from_fen(fen);
        let pseudo_moves = generate_pseudo_moves_fast(&pos);

        for mv in pseudo_moves.as_slice() {
            let old_result = {
                let mut test_pos = pos;
                let undo = test_pos.make_move(mv);
                let mut legal_test = test_pos;
                legal_test.side_to_move = pos.side_to_move;
                let result = !bitboard::movegen::is_in_check(&legal_test, pos.side_to_move);
                test_pos.unmake_move(mv, &undo);
                result
            };

            let new_result = is_move_legal_without_making(&pos, mv);

            assert_eq!(
                old_result, new_result,
                "Pin legality mismatch for move {:?} in position {}",
                mv, fen
            );
        }
    }
}
