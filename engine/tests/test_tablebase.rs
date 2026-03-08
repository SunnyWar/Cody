use bitboard::movegen::generate_legal_moves;
use bitboard::position::Position;

/// Optional integration test for local Syzygy installations.
///
/// Set `CODY_SYZYGY_PATH` to a valid tablebase directory (or ';'-separated
/// directories) to run this test.
#[test]
fn test_probe_root_best_move_with_local_syzygy() {
    let path = match std::env::var("CODY_SYZYGY_PATH") {
        Ok(p) if !p.trim().is_empty() => p,
        _ => return, // Skip when no local tablebase path is configured.
    };

    engine::search::tablebase::set_syzygy_path(&path)
        .expect("failed to load syzygy path from CODY_SYZYGY_PATH");

    // Simple 3-man position that is tablebase-resolved and should be winning
    // for White.
    let pos = Position::from_fen("6k1/8/8/8/8/8/5K2/6Q1 w - - 0 1");

    let root_score = engine::search::tablebase::probe_wdl_cp(&pos)
        .expect("expected WDL probe result for 3-man position");
    assert!(
        root_score > 0,
        "KQvK should probe as winning for side to move, got {root_score}"
    );

    let best_move = engine::search::tablebase::probe_root_best_move(&pos)
        .expect("expected tablebase best move at root");
    let legal_moves = generate_legal_moves(&pos);

    assert!(
        legal_moves.contains(&best_move),
        "tablebase best move must be legal, got {}",
        best_move
    );
}
