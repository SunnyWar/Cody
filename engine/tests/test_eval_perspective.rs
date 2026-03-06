use bitboard::movegen::SimpleMoveGen;
use bitboard::piece::Color;
use bitboard::position::Position;
use engine::Engine;
use engine::MaterialEvaluator;
use engine::search::evaluator::evaluate_for_side_to_move;

#[test]
fn test_side_to_move_pov_score_negates_when_stm_flips() {
    let pos_white = Position::from_fen("4k3/8/8/8/3Q4/8/4P3/4K3 w - - 0 1");
    let mut pos_black = pos_white;
    pos_black.side_to_move = Color::Black;

    let ev = MaterialEvaluator;
    let score_white = evaluate_for_side_to_move(&ev, &pos_white);
    let score_black = evaluate_for_side_to_move(&ev, &pos_black);

    assert_eq!(score_white, -score_black);
    assert!(score_white > 0, "White should be better in this position");
}

#[test]
fn test_depth0_search_uses_side_to_move_perspective() {
    let mut pos = Position::from_fen("4k3/8/8/8/3Q4/8/4P3/4K3 w - - 0 1");
    let mut engine = Engine::new(65_536, SimpleMoveGen, MaterialEvaluator);

    let (_, white_score) = engine.search(&pos, 0, None, None);

    pos.side_to_move = Color::Black;
    let (_, black_score) = engine.search(&pos, 0, None, None);

    assert!(white_score > 0, "White-to-move score should be positive");
    assert!(black_score < 0, "Black-to-move score should be negative");
    assert_eq!(white_score, -black_score);
}

#[test]
fn test_black_finds_forcing_tactical_capture() {
    // Black can win White's queen immediately with e5d4.
    let pos = Position::from_fen("4k3/8/8/4p3/3Q4/8/8/4K3 b - - 0 1");
    let mut engine = Engine::new(65_536, SimpleMoveGen, MaterialEvaluator);

    let (best_move, score) = engine.search(&pos, 2, None, None);

    assert_eq!(best_move.to_string(), "e5d4");
    assert!(
        score > 50,
        "Black should evaluate this as clearly favorable, got {}",
        score
    );
}
