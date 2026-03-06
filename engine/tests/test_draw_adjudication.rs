use bitboard::movegen::SimpleMoveGen;
use bitboard::position::Position;
use engine::Engine;
use engine::MaterialEvaluator;
use engine::core::arena::Arena;
use engine::core::tt::TranspositionTable;
use engine::search::INF;
use engine::search::MAX_REPETITION_HISTORY;
use engine::search::SearchHeuristics;
use engine::search::search_node_with_arena;

#[test]
fn test_fifty_move_rule_scored_as_draw() {
    // Bare kings with halfmove clock at 100 => claimable draw.
    let pos = Position::from_fen("7k/8/8/8/8/8/8/K7 w - - 100 1");
    let mut engine = Engine::new(65_536, SimpleMoveGen, MaterialEvaluator);

    let (_best_move, score) = engine.search(&pos, 2, None, None);
    assert_eq!(score, 0, "Expected draw score for 50-move position");
}

#[test]
fn test_threefold_repetition_scored_as_draw_in_search_node() {
    let mut arena = Arena::new(256);
    arena.get_mut(0).position = Position::default();

    let key = arena.get(0).position.zobrist_hash();
    let mut repetition_history = [0u64; MAX_REPETITION_HISTORY];
    repetition_history[0] = key;
    repetition_history[1] = key;
    repetition_history[2] = key;

    let mut tt = TranspositionTable::new(1);
    let mut heuristics = SearchHeuristics::new();

    let score = search_node_with_arena(
        &SimpleMoveGen,
        &MaterialEvaluator,
        &mut arena,
        0,
        3,
        -INF,
        INF,
        &mut tt,
        &mut heuristics,
        None,
        None,
        None,
        &mut repetition_history,
        3,
    );

    assert_eq!(score, 0, "Expected draw score for threefold repetition");
}
