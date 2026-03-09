use bitboard::movegen::SimpleMoveGen;
use bitboard::position::Position;
use engine::Engine;
use engine::MaterialEvaluator;

#[test]
fn test_krk_is_scored_as_clearly_winning() {
    // White has K+R vs bare king.
    let winning_fen = "8/8/8/4k3/8/8/8/K6R w - - 0 1";
    // Mirror: Black has K+R vs bare king.
    let losing_fen = "k6r/8/8/8/4K3/8/8/8 w - - 0 1";

    let winning = Position::from_fen(winning_fen);
    let losing = Position::from_fen(losing_fen);

    let mut engine = Engine::new(65_536, SimpleMoveGen, MaterialEvaluator);

    let (_, winning_score) = engine.search(&winning, 0, None, None);
    let (_, losing_score) = engine.search(&losing, 0, None, None);

    assert!(
        winning_score > 550,
        "expected clear KRK winning eval for side to move, got {winning_score}"
    );
    assert!(
        losing_score < -550,
        "expected clear KRK losing eval for side to move, got {losing_score}"
    );
}

#[test]
fn test_krk_corner_progress_scores_higher_than_center() {
    // Defender king in the center: conversion progress is early.
    let center_fen = "8/8/8/4k3/8/8/8/K6R w - - 0 1";
    // Defender king near corner with attacking king/rook coordinated.
    let corner_fen = "k7/1R6/2K5/8/8/8/8/8 w - - 0 1";

    let center = Position::from_fen(center_fen);
    let corner = Position::from_fen(corner_fen);

    let mut engine = Engine::new(65_536, SimpleMoveGen, MaterialEvaluator);

    let (_, center_score) = engine.search(&center, 0, None, None);
    let (_, corner_score) = engine.search(&corner, 0, None, None);

    assert!(
        corner_score > center_score + 120,
        "expected cornered KRK position to score significantly higher (center={center_score}, \
         corner={corner_score})"
    );
}
