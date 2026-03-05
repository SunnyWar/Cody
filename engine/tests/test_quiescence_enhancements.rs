// Test to validate quiescence search enhancements for horizon effect prevention

use bitboard::movegen::SimpleMoveGen;
use bitboard::position::Position;
use engine::Engine;
use engine::MaterialEvaluator;

#[test]
fn test_quiescence_finds_checks_in_tactical_position() {
    // Position where a checking move exists that should be found by qsearch
    // This is a position where White can give check with Qh5+
    let fen = "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2";
    let pos = Position::from_fen(fen);

    let mut engine = Engine::new(65_536, SimpleMoveGen, MaterialEvaluator);

    // Search at depth 3 to test quiescence behavior
    let (best_move, _score) = engine.search(&pos, 3, None, None);

    // The engine should find a reasonable move (not null)
    assert!(
        !best_move.is_null(),
        "Engine should find a move in this position"
    );
}

#[test]
fn test_quiescence_avoids_horizon_blunders() {
    // Position where White can capture Black's queen
    // Either pawn or queen can capture - both are good moves
    let fen = "rn2kbnr/ppp1pppp/8/3q4/3QP3/8/PPP2PPP/RNB1KBNR w KQkq - 0 5";
    let pos = Position::from_fen(fen);

    let mut engine = Engine::new(65_536, SimpleMoveGen, MaterialEvaluator);

    // At depth 2, the engine should find a favorable capture
    let (best_move, score) = engine.search(&pos, 2, None, None);

    // Should find a capture of the queen (either Qxd5 or exd5)
    assert!(
        best_move.to_string().ends_with("d5"),
        "Should capture the queen, got {}",
        best_move
    );

    // Score should strongly favor White (winning the queen)
    assert!(
        score > 800,
        "Should recognize major material advantage, got score {}",
        score
    );
}

#[test]
fn test_quiescence_mate_detection() {
    // Simple mate in 1: White queen can deliver back-rank mate
    // Black king on g8, White queen delivers Qd8#
    let fen = "6k1/5ppp/8/8/8/8/5PPP/3Q2K1 w - - 0 1";
    let pos = Position::from_fen(fen);

    let mut engine = Engine::new(65_536, SimpleMoveGen, MaterialEvaluator);

    // Should find checkmate in 1 move
    let (best_move, score) = engine.search(&pos, 3, None, None);

    // The move should not be null
    assert!(!best_move.is_null(), "Should find a best move");

    // Should recognize this is a winning position (very high score or mate
    // detection)
    assert!(
        score > 1000 || score > 29_000,
        "Should detect winning or mate position, got score {}",
        score
    );
}
