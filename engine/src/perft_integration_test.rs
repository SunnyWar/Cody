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

        // Ensure the illegal move reported in the log is not generated.
        let illegal_from = Square::from_coords('a', '6').unwrap();
        let illegal_to = Square::from_coords('h', '4').unwrap();
        let illegal_present = generate_legal_moves(&pos)
            .iter()
            .any(|m| m.from() == illegal_from && m.to() == illegal_to);
        assert!(!illegal_present, "Illegal move a6h4 was generated");

        let count = perft(&pos, 1);
        println!("Midgame position before illegal move: perft(1) = {}", count);
        assert!(count > 0, "Should have legal moves for Black");
    }

    #[test]
    fn test_perft_midgame_position_white_illegal_promotion() {
        let fen = "rn1qnrk1/1P3p2/6pp/1p4bP/PpB1p1P1/1P2PN2/2Q2P2/R1B2RK1 w - - 0 12";
        let pos = Position::from_fen(fen);

        // Ensure the illegal move reported in the log is not generated.
        let illegal_from = Square::from_coords('b', '7').unwrap();
        let illegal_to = Square::from_coords('a', '8').unwrap();
        let moves = generate_legal_moves(&pos);
        let illegal_present = moves
            .iter()
            .any(|m| m.from() == illegal_from && m.to() == illegal_to && m.promotion().is_none());
        assert!(
            !illegal_present,
            "Illegal move b7a8 without promotion was generated"
        );

        let promo_targets = moves
            .iter()
            .filter(|m| m.from() == illegal_from && m.to() == illegal_to)
            .map(|m| m.promotion())
            .collect::<Vec<_>>();
        for promo in [
            bitboard::piece::PieceKind::Queen,
            bitboard::piece::PieceKind::Rook,
            bitboard::piece::PieceKind::Bishop,
            bitboard::piece::PieceKind::Knight,
        ] {
            assert!(
                promo_targets.contains(&Some(promo)),
                "Missing promotion move b7a8{}",
                promo.to_uci()
            );
        }

        let count = perft(&pos, 1);
        println!("Midgame position before illegal move: perft(1) = {}", count);
        assert!(count > 0, "Should have legal moves for White");
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

        // Ensure the illegal move d1e1 is not made
        let illegal_from = Square::from_coords('d', '1').unwrap();
        let illegal_to = Square::from_coords('e', '1').unwrap();
        let illegal_present = generate_legal_moves(&pos)
            .iter()
            .any(|m| m.from() == illegal_from && m.to() == illegal_to);
        assert!(!illegal_present, "Illegal move d1e1 was generated");

        let count = perft(&pos, 1);
        println!("Position with two queens: perft(1) = {}", count);
        assert!(count > 0, "Should have legal moves for White");
    }

    #[test]
    fn test_promotion_move_consistency() {
        // FEN just before Black plays c2xd1=Q (capture pawn on c2 takes rook on d1 and
        // promotes to queen)
        let fen = "r1bq1rk1/pp2bppp/2n5/3p4/P2P4/4B1P1/1PpNPP1P/R2Q1RKB b - - 0 3";
        let pos = Position::from_fen(fen);
        let original_hash = pos.zobrist_hash();

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
}
