/// Integration test demonstrating perft usage
/// Run with: cargo test --lib test_perft_integration -- --nocapture
#[cfg(test)]
mod perft_integration_tests {
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
}
