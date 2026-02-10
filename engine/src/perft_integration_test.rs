/// Integration test demonstrating perft usage
/// Run with: cargo test --lib test_perft_integration -- --nocapture
#[cfg(test)]
mod perft_integration_tests {
    use bitboard::Square;
    use bitboard::movegen::generate_legal_moves;
    use bitboard::perft;
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
}
