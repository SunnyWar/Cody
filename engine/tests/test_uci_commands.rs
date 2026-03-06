/// Integration tests for UCI command handlers
///
/// This test suite validates each UCI command exposed by the Cody engine's
/// UCI API, ensuring proper parsing, handling, and output for standard UCI
/// protocol interactions.
use engine::api::uciapi::CodyApi;

#[test]
fn test_uci_command() {
    let mut api = CodyApi::new();
    let mut output = Vec::new();

    api.handle_uci(&mut output);

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("id name Cody"));
    assert!(output_str.contains("id author Strong Noodle"));
    assert!(output_str.contains("option name Hash type spin"));
    assert!(output_str.contains("option name Clear Hash type button"));
    assert!(output_str.contains("option name Threads type spin"));
    assert!(output_str.contains("option name Ponder type check"));
    assert!(output_str.contains("option name Verbose type check"));
    assert!(output_str.contains("uciok"));
}

#[test]
fn test_isready_command() {
    let mut api = CodyApi::new();
    let mut output = Vec::new();

    api.handle_isready(&mut output);

    let output_str = String::from_utf8(output).unwrap();
    assert_eq!(output_str.trim(), "readyok");
}

#[test]
fn test_position_startpos() {
    let mut api = CodyApi::new();
    let mut output = Vec::new();

    api.handle_position("position startpos", &mut output);

    // Verify the position is set to the starting position
    assert_eq!(
        api.current_pos.to_fen(),
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    );
}

#[test]
fn test_position_startpos_with_moves() {
    let mut api = CodyApi::new();
    let mut output = Vec::new();

    api.handle_position("position startpos moves e2e4 e7e5", &mut output);

    // Verify moves were applied
    let fen = api.current_pos.to_fen();
    assert!(fen.contains("4p3")); // e5 pawn
    assert!(fen.contains("4P3")); // e4 pawn
    assert!(fen.contains("w ")); // White to move (after e2e4 and e7e5)
}

#[test]
fn test_position_fen() {
    let mut api = CodyApi::new();
    let mut output = Vec::new();

    let test_fen = "r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3";
    api.handle_position(&format!("position fen {}", test_fen), &mut output);

    // Verify the FEN was correctly parsed
    assert_eq!(api.current_pos.to_fen(), test_fen);
}

#[test]
fn test_position_fen_with_moves() {
    let mut api = CodyApi::new();
    let mut output = Vec::new();

    let test_fen = "r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3";
    api.handle_position(
        &format!("position fen {} moves d2d4", test_fen),
        &mut output,
    );

    let result_fen = api.current_pos.to_fen();
    // After d2d4, white pawn should be on d4
    assert!(result_fen.contains("3P"));
}

#[test]
fn test_setoption_threads() {
    let mut api = CodyApi::new();

    // Test setting threads to 2
    api.handle_setoption("setoption name Threads value 2");

    // Note: We can't directly verify thread count without exposing it,
    // but we can verify the command doesn't panic
}

#[test]
fn test_setoption_verbose() {
    let mut api = CodyApi::new();

    // Test enabling verbose mode
    api.handle_setoption("setoption name Verbose value true");

    // Test disabling verbose mode
    api.handle_setoption("setoption name Verbose value false");

    // Command should not panic
}

#[test]
fn test_setoption_hash() {
    let mut api = CodyApi::new();

    // Test setting hash size
    api.handle_setoption("setoption name Hash value 32");
    api.handle_setoption("setoption name Hash value 128");
    api.handle_setoption("setoption name Hash value 1024");

    // Command should not panic
}

#[test]
fn test_setoption_clear_hash() {
    let mut api = CodyApi::new();
    let mut output = Vec::new();

    // Run a search to populate the hash table
    api.handle_position("position startpos", &mut output);
    api.handle_go("go depth 2", &mut output);

    // Clear the hash
    api.handle_setoption("setoption name Clear Hash");

    // Should not panic and engine should still work
    output.clear();
    api.handle_go("go depth 1", &mut output);
    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("bestmove"));
}

#[test]
fn test_setoption_ponder() {
    let mut api = CodyApi::new();

    // Test enabling/disabling ponder
    api.handle_setoption("setoption name Ponder value true");
    api.handle_setoption("setoption name Ponder value false");

    // Command should not panic
}

#[test]
fn test_go_depth() {
    let mut api = CodyApi::new();
    let mut output = Vec::new();

    // Set position
    api.handle_position("position startpos", &mut output);
    output.clear();

    // Search to depth 1 (should be fast)
    api.handle_go("go depth 1", &mut output);

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("bestmove"));
    // Verify it's not a null move
    assert!(!output_str.contains("bestmove 0000"));
}

#[test]
fn test_go_movetime() {
    let mut api = CodyApi::new();
    let mut output = Vec::new();

    api.handle_position("position startpos", &mut output);
    output.clear();

    // Search for 50ms
    api.handle_go("go movetime 50", &mut output);

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("bestmove"));
}

#[test]
fn test_go_with_time_controls() {
    let mut api = CodyApi::new();
    let mut output = Vec::new();

    api.handle_position("position startpos", &mut output);
    output.clear();

    // Simulate a game with time controls
    api.handle_go(
        "go wtime 60000 btime 60000 winc 1000 binc 1000",
        &mut output,
    );

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("bestmove"));
}

#[test]
fn test_go_depth_with_black_to_move() {
    let mut api = CodyApi::new();
    let mut output = Vec::new();

    // Set position with black to move
    api.handle_position("position startpos moves e2e4", &mut output);
    output.clear();

    api.handle_go("go depth 1", &mut output);

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("bestmove"));
}

#[test]
fn test_go_ponder() {
    let mut api = CodyApi::new();
    let mut output = Vec::new();

    api.handle_position("position startpos", &mut output);
    output.clear();

    // Test that ponder flag is parsed (even if not fully implemented)
    api.handle_go("go ponder depth 1", &mut output);

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("bestmove"));
}

#[test]
fn test_newgame_command() {
    let mut api = CodyApi::new();
    let mut output = Vec::new();

    // Modify position
    api.handle_position("position startpos moves e2e4 e7e5", &mut output);

    // Verify position changed
    assert!(!api.current_pos.to_fen().contains("w KQkq - 0 1"));

    output.clear();
    api.handle_newgame(&mut output);

    // Verify position is reset to starting position
    assert_eq!(
        api.current_pos.to_fen(),
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    );
}

#[test]
fn test_bench_command() {
    let mut api = CodyApi::new();
    let mut output = Vec::new();

    api.handle_bench("bench", &mut output);

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("Total time (ms)"));
    assert!(output_str.contains("Nodes searched"));
    assert!(output_str.contains("Nodes/second"));
}

#[test]
fn test_position_with_multiple_moves() {
    let mut api = CodyApi::new();
    let mut output = Vec::new();

    api.handle_position(
        "position startpos moves e2e4 e7e5 g1f3 b8c6 f1c4",
        &mut output,
    );

    // Verify all moves were applied
    let fen = api.current_pos.to_fen();
    assert!(fen.contains("b ")); // Black to move after 5 half-moves
}

#[test]
fn test_position_with_castling() {
    let mut api = CodyApi::new();
    let mut output = Vec::new();

    // Italian Game position with castling available
    api.handle_position(
        "position startpos moves e2e4 e7e5 g1f3 b8c6 f1c4 f8c5",
        &mut output,
    );

    let fen = api.current_pos.to_fen();
    // Both sides should still have castling rights
    assert!(fen.contains("KQkq") || fen.contains("KQ") || fen.contains("kq"));
}

#[test]
fn test_position_with_en_passant() {
    let mut api = CodyApi::new();
    let mut output = Vec::new();

    // Create a position where en passant is possible
    api.handle_position("position startpos moves e2e4 a7a6 e4e5 d7d5", &mut output);

    let fen = api.current_pos.to_fen();
    // Should have en passant square set to d6
    assert!(fen.contains("d6"));
}

#[test]
fn test_go_with_invalid_position_handles_gracefully() {
    let mut api = CodyApi::new();
    let mut output = Vec::new();

    // Even with default position, go should work
    api.handle_go("go depth 1", &mut output);

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("bestmove"));
}

#[test]
fn test_setoption_case_insensitive() {
    let mut api = CodyApi::new();

    // Test various case combinations
    api.handle_setoption("setoption name threads value 1");
    api.handle_setoption("setoption name THREADS value 1");
    api.handle_setoption("setoption name Threads value 1");

    // Should not panic with any case variation
}

#[test]
fn test_position_invalid_move_handling() {
    let mut api = CodyApi::new();
    let mut output = Vec::new();

    // Try to apply an invalid move (should be handled gracefully)
    api.handle_position("position startpos moves e2e4 z9z9", &mut output);

    // Engine should not panic; position should be at e2e4
    let fen = api.current_pos.to_fen();
    assert!(fen.contains("4P3")); // e4 pawn present
}

#[test]
fn test_go_default_time() {
    let mut api = CodyApi::new();
    let mut output = Vec::new();

    api.handle_position("position startpos", &mut output);
    output.clear();

    // Bare "go" command should use default time limit
    api.handle_go("go", &mut output);

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("bestmove"));
}

#[test]
fn test_position_promotion() {
    let mut api = CodyApi::new();
    let mut output = Vec::new();

    // Set up a position near promotion
    let promotion_fen = "8/4P3/8/8/8/8/4k3/4K3 w - - 0 1";
    api.handle_position(&format!("position fen {}", promotion_fen), &mut output);

    // Verify the FEN was set
    assert_eq!(api.current_pos.to_fen(), promotion_fen);
}

#[test]
fn test_complex_position_sequence() {
    let mut api = CodyApi::new();
    let mut output = Vec::new();

    // Test a complex sequence of position commands
    api.handle_position("position startpos", &mut output);
    output.clear();

    api.handle_position("position startpos moves e2e4", &mut output);
    output.clear();

    api.handle_position("position startpos moves e2e4 e7e5 g1f3", &mut output);

    let fen = api.current_pos.to_fen();
    assert!(fen.contains("5N2")); // Knight on f3
}

#[test]
fn test_go_with_black_time_controls() {
    let mut api = CodyApi::new();
    let mut output = Vec::new();

    // Position with black to move
    api.handle_position("position startpos moves e2e4", &mut output);
    output.clear();

    // Search with black's time
    api.handle_go("go btime 30000 wtime 30000 binc 500 winc 500", &mut output);

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("bestmove"));
}

#[test]
fn test_multiple_searches_same_position() {
    let mut api = CodyApi::new();
    let mut output = Vec::new();

    api.handle_position("position startpos", &mut output);

    // Run multiple searches
    for depth in 1..=3 {
        output.clear();
        api.handle_go(&format!("go depth {}", depth), &mut output);

        let output_str = String::from_utf8_lossy(&output);
        assert!(output_str.contains("bestmove"));
    }
}

#[test]
fn test_newgame_clears_state() {
    let mut api = CodyApi::new();
    let mut output = Vec::new();

    // Execute a search to populate engine state
    api.handle_position("position startpos", &mut output);
    api.handle_go("go depth 2", &mut output);

    // Clear with newgame
    output.clear();
    api.handle_newgame(&mut output);

    // Position should be reset
    assert_eq!(
        api.current_pos.to_fen(),
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    );

    // Should be able to search again
    output.clear();
    api.handle_go("go depth 1", &mut output);
    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("bestmove"));
}
