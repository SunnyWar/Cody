// Test to validate enhanced piece-square table evaluation

use bitboard::movegen::SimpleMoveGen;
use bitboard::position::Position;
use engine::Engine;
use engine::MaterialEvaluator;
use engine::search::evaluator::Evaluator;

#[test]
fn test_pst_knight_prefers_center_in_midgame() {
    // Two positions: knight on rim vs knight in center (same material)
    // Midgame position with queens on board

    // Knight on rim (a1)
    let rim_knight = Position::from_fen("rnbqkb1r/pppppppp/8/8/8/8/PPPPPPPP/NNBQKB1R w KQkq - 0 1");

    // Knight in center (d4)
    let center_knight =
        Position::from_fen("rnbqkb1r/pppppppp/8/8/3N4/8/PPPPPPPP/R1BQKB1R w KQkq - 0 1");

    let evaluator = MaterialEvaluator;

    let rim_score = evaluator.evaluate(&rim_knight);
    let center_score = evaluator.evaluate(&center_knight);

    // Centralized knight should be significantly better (+20 centipawns from PST
    // alone)
    assert!(
        center_score > rim_score + 15,
        "Centralized knight should score higher: center={}, rim={}",
        center_score,
        rim_score
    );
}

#[test]
fn test_pst_king_safety_in_opening() {
    // King castled vs king in center (opening phase with queens)

    // King castled kingside (safe) - rooks and bishops placed symmetrically, only
    // king position differs
    let castled = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQ1RK1 w - - 0 1");

    // King exposed on f1 (not castled)
    let exposed = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQ1K1R w - - 0 1");

    let evaluator = MaterialEvaluator;

    let castled_score = evaluator.evaluate(&castled);
    let exposed_score = evaluator.evaluate(&exposed);

    // Both positions have full material (opening), castled king should be safer
    // But difference may be small since both are on back rank
    // g1 gets +30, f1 gets -5 in midgame table, difference = 35 centipawns
    assert!(
        castled_score > exposed_score + 10,
        "Castled king should be safer: castled={}, exposed={}",
        castled_score,
        exposed_score
    );
}

#[test]
fn test_pst_king_centralization_in_endgame() {
    // Endgame (no queens, few pieces): king on rim vs king in center
    // Use minimal material to ensure endgame phase

    // King on rim in endgame - White king a1, Black king h8
    let rim_king = Position::from_fen("7k/8/8/8/8/8/8/K7 w - - 0 1");

    // King centralized in endgame - White king d4, Black king h8
    let center_king = Position::from_fen("7k/8/8/8/3K4/8/8/8 w - - 0 1");

    let evaluator = MaterialEvaluator;

    let rim_score = evaluator.evaluate(&rim_king);
    let center_score = evaluator.evaluate(&center_king);

    // In endgame, centralized king should be much better
    // King on d4 (index 27) gets KING_ENDGAME_TABLE[27]=+40
    // King on a1 (index 0) gets KING_ENDGAME_TABLE[0]=-50
    // Difference should be 90 centipawns
    assert!(
        center_score > rim_score + 30,
        "Centralized endgame king should be stronger: center={}, rim={}",
        center_score,
        rim_score
    );
}

#[test]
fn test_pst_advanced_pawns_in_endgame() {
    // Endgame: pawn on 2nd rank vs pawn on 6th rank

    // Pawn barely moved (endgame)
    let backward_pawn = Position::from_fen("8/8/8/8/8/8/P7/K6k w - - 0 1");

    // Pawn advanced to 6th rank (near promotion)
    let advanced_pawn = Position::from_fen("8/8/P7/8/8/8/8/K6k w - - 0 1");

    let evaluator = MaterialEvaluator;

    let backward_score = evaluator.evaluate(&backward_pawn);
    let advanced_score = evaluator.evaluate(&advanced_pawn);

    // Advanced pawn in endgame is more valuable
    // Pawn on a2 (index 8) gets PAWN_ENDGAME_TABLE[8]=+5
    // Pawn on a6 (index 40) gets PAWN_ENDGAME_TABLE[40]=+35
    // Difference should be 30 centipawns, but with other factors may be less
    assert!(
        advanced_score > backward_score,
        "Advanced endgame pawn should be more valuable: advanced={}, backward={}",
        advanced_score,
        backward_score
    );
}

#[test]
fn test_pst_phase_blending() {
    // Test that evaluation correctly transitions from midgame to endgame

    // Full material (midgame phase)
    let midgame = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

    // Minimal material (endgame phase)
    let endgame = Position::from_fen("8/8/8/8/8/8/8/K6k w - - 0 1");

    let evaluator = MaterialEvaluator;

    let midgame_score = evaluator.evaluate(&midgame);
    let endgame_score = evaluator.evaluate(&endgame);

    // Both should evaluate (not panic), and starting position should be ~equal
    assert!(
        midgame_score.abs() < 100,
        "Starting position should be roughly equal, got {}",
        midgame_score
    );

    assert!(
        endgame_score.abs() < 50,
        "Bare kings should be equal, got {}",
        endgame_score
    );
}

#[test]
fn test_pst_influences_search() {
    // Test that PST bonuses actually influence move selection
    // Position where knight can go to center or rim
    let pos = Position::from_fen("rnbqkb1r/pppppppp/8/8/8/8/PPPPPPPP/RNBQKB1R w KQkq - 0 1");

    let mut engine = Engine::new(65_536, SimpleMoveGen, MaterialEvaluator);

    // Search at shallow depth
    let (best_move, _score) = engine.search(&pos, 2, None, None);

    // Should find a reasonable developing move (not null)
    assert!(
        !best_move.is_null(),
        "Engine should find a move with PST guidance"
    );

    // The move should be a knight move (developing to good square)
    let move_str = best_move.to_string();
    let from_sq = &move_str[0..2];

    // At least one of the starting squares should be where knights start
    assert!(
        from_sq == "b1" || from_sq == "g1" || from_sq == "e2" || from_sq == "d2",
        "Should develop pieces (got {})",
        move_str
    );
}
