/// Integration test demonstrating perft usage
/// Run with: cargo test --lib test_perft_integration -- --nocapture
#[cfg(test)]
mod perft_integration_tests {
    use bitboard::Square;
    use bitboard::movegen::generate_legal_moves;
    use bitboard::perft;
    use bitboard::piece::Color;
    use bitboard::piece::Piece;
    use bitboard::position::Position;

    #[test]
    fn test_perft_initial_position_depth_2() {
        let pos = Position::default();
        let count = perft(&pos, 2);
        println!("Initial position: perft(2) = {}", count);
        assert!(count > 0, "Should have legal moves");
    }

    #[test]
    fn test_perft_divide_example() {
        let pos = Position::default();
        println!("\nInitial position move breakdown (depth 1):");

        // At depth 1, each move represents itself
        let moves = bitboard::movegen::generate_legal_moves(&pos);
        println!("Total moves: {}", moves.len());

        for (i, mv) in moves.iter().take(5).enumerate() {
            println!("  Move {}: {} -> {}", i + 1, mv.from, mv.to);
        }
    }

    #[test]
    fn test_perft_kiwipete_position() {
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 10";
        let pos = Position::from_fen(fen);

        // Known values for Kiwipete from various sources
        assert_eq!(perft(&pos, 1), 48);
        println!("Kiwipete: perft(1) = 48 ✓");
    }

    #[test]
    fn test_perft_endgame_position() {
        let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 11";
        let pos = Position::from_fen(fen);

        // Simple endgame position
        let count = perft(&pos, 1);
        println!("Simple endgame: perft(1) = {}", count);
        assert!(count > 0);
    }

    #[test]
    fn test_perft_midgame_position() {
        let fen = "r1b1r1k1/1p1nqppp/p1pb1n2/3p4/2PPp2P/1PN1PNP1/P1Q1BP2/R1BR2K1 b - - 0 3";
        let pos = Position::from_fen(fen);

        let mut moves: Vec<String> = generate_legal_moves(&pos)
            .iter()
            .map(|m| m.to_string())
            .collect();
        moves.sort();

        let mut expected = vec![
            "a6a5", "c6c5", "b7b6", "g7g6", "h7h6", "b7b5", "g7g5", "h7h5", "d5c4", "e4f3", "f6g4",
            "f6h5", "d7c5", "d7e5", "d7b6", "d7b8", "d7f8", "d6a3", "d6g3", "d6b4", "d6f4", "d6c5",
            "d6e5", "d6c7", "d6b8", "a8a7", "a8b8", "e8d8", "e8f8", "e7e5", "e7e6", "e7d8", "e7f8",
            "g8f8", "g8h8",
        ];
        expected.sort();

        assert_eq!(
            moves, expected,
            "Unexpected legal moves in midgame position"
        );
    }

    #[test]
    fn test_perft_midgame_position_white_illegal_promotion() {
        let fen = "rn1qnrk1/1P3p2/6pp/1p4bP/PpB1p1P1/1P2PN2/2Q2P2/R1B2RK1 w - - 0 12";
        let pos = Position::from_fen(fen);

        let mut moves: Vec<String> = generate_legal_moves(&pos)
            .iter()
            .map(|m| m.to_string())
            .collect();
        moves.sort();

        let mut expected = vec![
            "a4a5", "b7a8q", "b7a8r", "b7a8b", "b7a8n", "a4b5", "h5g6", "f3e1", "f3d2", "f3h2",
            "f3d4", "f3h4", "f3e5", "f3g5", "c1b2", "c1d2", "c1a3", "c4e2", "c4d3", "c4b5", "c4d5",
            "c4e6", "c4f7", "a1b1", "a1a2", "a1a3", "f1d1", "f1e1", "c2b1", "c2d1", "c2a2", "c2b2",
            "c2d2", "c2e2", "c2c3", "c2d3", "c2e4", "g1h1", "g1g2", "g1h2",
        ];
        expected.sort();

        assert_eq!(
            moves, expected,
            "Unexpected legal moves in midgame promotion position"
        );
    }

    #[test]
    fn test_perft_limited_legal_moves_position() {
        let fen = "3qk2r/3P3p/rnn1p2P/2b3p1/2B2NP1/4pP2/1R6/1qB1K2b b k - 0 20";
        let pos = Position::from_fen(fen);

        let mut moves: Vec<String> = generate_legal_moves(&pos)
            .iter()
            .map(|m| m.to_string())
            .collect();
        moves.sort();

        let mut expected = vec!["b6d7", "d8d7", "e8d7", "e8e7", "e8f7", "e8f8"];
        expected.sort();

        assert_eq!(moves, expected, "Unexpected legal moves in position");
    }

    #[test]
    fn test_perft_two_queens_illegal_move() {
        let fen = "r1bq1rk1/1p4pp/p7/N2p1p2/P2P1P2/bP2B1PP/4P3/R2q1RKB w - - 0 9";
        let pos = Position::from_fen(fen);

        let mut moves: Vec<String> = generate_legal_moves(&pos)
            .iter()
            .map(|m| m.to_string())
            .collect();
        moves.sort();

        let mut expected = vec![
            "b3b4", "g3g4", "h3h4", "a5c4", "a5c6", "a5b7", "h1g2", "h1f3", "h1e4", "h1d5", "e3c1",
            "e3d2", "e3f2", "a1b1", "a1c1", "a1d1", "a1a2", "a1a3", "f1d1", "f1e1", "g1h2", "g1g2",
            "g1f2",
        ];
        expected.sort();

        assert_eq!(
            moves, expected,
            "Unexpected legal moves in two-queens position"
        );
    }

    #[test]
    fn test_promotion_move_consistency() {
        // FEN just before Black plays c2xd1=Q (capture pawn on c2 takes rook on d1 and
        // promotes to queen)
        let fen = "r1bq1rk1/pp2bppp/2n5/3p4/P2P4/4B1P1/1PpNPP1P/R2Q1RKB b - - 0 3";
        let pos = Position::from_fen(fen);
        let original_hash = pos.zobrist_hash();

        let mut moves: Vec<String> = generate_legal_moves(&pos)
            .iter()
            .map(|m| m.to_string())
            .collect();
        moves.sort();

        let mut expected = vec![
            "a7a6", "b7b6", "f7f6", "g7g6", "h7h6", "a7a5", "b7b5", "f7f5", "g7g5", "h7h5",
            "c2d1q", "c2d1r", "c2d1b", "c2d1n", "c2c1q", "c2c1r", "c2c1b", "c2c1n", "c6b4", "c6d4",
            "c6a5", "c6e5", "c6b8", "e7a3", "e7b4", "e7h4", "e7c5", "e7g5", "e7d6", "e7f6", "c8h3",
            "c8g4", "c8f5", "c8e6", "c8d7", "a8b8", "f8e8", "d8a5", "d8b6", "d8d6", "d8c7", "d8d7",
            "d8e8", "g8h8",
        ];
        expected.sort();

        assert_eq!(
            moves, expected,
            "Unexpected legal moves in promotion consistency position"
        );

        // Parse the promotion move (capture + promote)
        let m_str = "c2d1q";
        let m = pos
            .parse_uci_move(m_str)
            .expect("Failed to parse promotion move");
        assert_eq!(m.from(), Square::C2, "Move from should be c2");
        assert_eq!(m.to(), Square::D1, "Move to should be d1");
        assert_eq!(
            m.promotion(),
            Some(bitboard::piece::PieceKind::Queen),
            "Move should be a queen promotion"
        );

        // Execute the promotion
        let mut new_pos = Position::default();
        pos.apply_move_into(&m, &mut new_pos);

        // Verify hash changed after the move
        let new_hash = new_pos.zobrist_hash();
        assert_ne!(
            new_hash, original_hash,
            "Hash should change after making a move"
        );

        // Verify the promoted piece is on d1 and is a Black Queen
        let black_queen = Piece::from_parts(Color::Black, Some(bitboard::piece::PieceKind::Queen));
        assert!(
            new_pos.pieces.get(black_queen).contains(Square::D1),
            "Promoted Black Queen not found on d1"
        );

        // Simulate undo by reconstructing from FEN and verifying consistency
        let reconstructed_pos = Position::from_fen(&pos.to_fen());
        assert_eq!(
            reconstructed_pos.zobrist_hash(),
            original_hash,
            "Reconstructed position hash should match original"
        );
        assert_eq!(
            reconstructed_pos.to_fen(),
            pos.to_fen(),
            "Reconstructed position FEN should match original"
        );
    }

    #[test]
    fn test_illegal_move_not_generated_in_complex_position() {
        // Position where d1 is a BLACK QUEEN and d1-e1 should NOT be legal for White
        let game_fen = "r1bq1rk1/1p4pp/p7/N2p1p2/P2P1P2/bP2B1PP/4P3/R2q1RKB w - - 0 9";
        let pos_with_black_queen = Position::from_fen(game_fen);

        let mut moves: Vec<String> = generate_legal_moves(&pos_with_black_queen)
            .iter()
            .map(|m| m.to_string())
            .collect();
        moves.sort();

        let mut expected = vec![
            "b3b4", "g3g4", "h3h4", "a5c4", "a5c6", "a5b7", "h1g2", "h1f3", "h1e4", "h1d5", "e3c1",
            "e3d2", "e3f2", "a1b1", "a1c1", "a1d1", "a1a2", "a1a3", "f1d1", "f1e1", "g1h2", "g1g2",
            "g1f2",
        ];
        expected.sort();

        assert_eq!(
            moves, expected,
            "Unexpected legal moves in complex position"
        );
    }
}
