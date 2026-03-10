// ...existing code...
use crate::VERBOSE;
use crate::api::uciapi::CodyApi;
use crate::search::evaluator::MaterialEvaluator;
use bitboard::Square;
use std::sync::atomic::Ordering;

#[test]
fn test_parse_go_limits_ponder_is_infinite_without_time_or_depth() {
    let api = CodyApi::new(MaterialEvaluator::default());
    let limits = api.parse_go_limits("go ponder");

    assert!(limits.ponder);
    assert!(limits.infinite);
    assert!(limits.movetime_ms.is_none());
    assert!(limits.depth.is_none());
}

#[test]
fn test_parse_go_limits_bare_go_keeps_default_movetime() {
    let api = CodyApi::new(MaterialEvaluator::default());
    let limits = api.parse_go_limits("go");

    assert_eq!(limits.movetime_ms, Some(1000));
    assert!(!limits.ponder);
}

#[test]
fn test_command_keyword_uses_exact_first_token() {
    assert_eq!(
        CodyApi::<MaterialEvaluator>::command_keyword("go depth 4"),
        Some("go")
    );
    assert_eq!(
        CodyApi::<MaterialEvaluator>::command_keyword("goo"),
        Some("goo")
    );
    assert_eq!(
        CodyApi::<MaterialEvaluator>::command_keyword("   help   "),
        Some("help")
    );
    assert_eq!(
        CodyApi::<MaterialEvaluator>::command_keyword("   \t   "),
        None
    );
}

#[test]
fn test_handle_help_lists_allowed_commands() {
    let api = &mut CodyApi::new(MaterialEvaluator::default());
    let mut out = Vec::<u8>::new();

    api.handle_help(&mut out);
    let text = String::from_utf8(out).expect("help output should be valid utf-8");

    assert!(text.contains("Allowed commands:"));
    assert!(text.contains("  help"));
    assert!(text.contains("  go [depth N|movetime MS|wtime|btime|winc|binc|ponder|infinite]"));
}

#[test]
fn test_dispatch_register_and_register_later_acknowledged() {
    let api = &mut CodyApi::new(MaterialEvaluator::default());
    let mut out = Vec::<u8>::new();

    let should_quit = api.dispatch_command("register", &mut out);
    assert!(!should_quit);
    let text = String::from_utf8(out).expect("register output should be valid utf-8");
    assert!(text.contains("info string registration not required"));

    let mut out_later = Vec::<u8>::new();
    let should_quit_later = api.dispatch_command("register later", &mut out_later);
    assert!(!should_quit_later);
    let text_later =
        String::from_utf8(out_later).expect("register later output should be valid utf-8");
    assert!(text_later.contains("info string registration not required"));
}

#[test]
fn test_dispatch_unknown_command_emits_message() {
    let api = &mut CodyApi::new(MaterialEvaluator::default());
    let mut out = Vec::<u8>::new();

    let should_quit = api.dispatch_command("not_a_real_command", &mut out);
    assert!(!should_quit);

    let text = String::from_utf8(out).expect("unknown command output should be valid utf-8");
    assert!(text.contains("Unknown command: 'not_a_real_command'"));
    assert!(text.contains("Type help for more information"));
}

#[test]
fn test_dispatch_quit_returns_true() {
    let api = &mut CodyApi::new(MaterialEvaluator::default());
    let mut out = Vec::<u8>::new();

    let should_quit = api.dispatch_command("quit", &mut out);
    assert!(should_quit);
    assert!(out.is_empty());
}

#[test]
fn test_dispatch_stop_sets_stop_flag() {
    let api = &mut CodyApi::new(MaterialEvaluator::default());
    let mut out = Vec::<u8>::new();

    assert!(!api.stop_requested());
    let should_quit = api.dispatch_command("stop", &mut out);

    assert!(!should_quit);
    assert!(api.stop_requested());
    assert!(out.is_empty());
}

#[test]
fn test_dispatch_ponderhit_clears_stop_and_ponder_flags() {
    let api = &mut CodyApi::new(MaterialEvaluator::default());
    let mut out = Vec::<u8>::new();

    // Seed internal state so we can validate the reset behavior.
    api.handle_setoption("setoption name Ponder value true");
    api.dispatch_command("go ponder depth 1", &mut out);
    assert!(api.current_limits().ponder);
    api.dispatch_command("stop", &mut out);
    assert!(api.stop_requested());

    let should_quit = api.dispatch_command("ponderhit", &mut out);
    assert!(!should_quit);
    assert!(!api.stop_requested());
    assert!(!api.current_limits().ponder);
    assert!(!api.current_limits().infinite);
}

#[test]
fn test_parse_go_limits_uses_white_time_budget_when_white_to_move() {
    let api = CodyApi::new(MaterialEvaluator::default());
    let limits = api.parse_go_limits("go wtime 60000 btime 30000 winc 1000 binc 500");

    assert_eq!(limits.movetime_ms, Some(2100));
    assert_eq!(limits.wtime_ms, Some(60000));
    assert_eq!(limits.btime_ms, Some(30000));
}

#[test]
fn test_parse_go_limits_uses_black_time_budget_when_black_to_move() {
    let api = &mut CodyApi::new(MaterialEvaluator::default());
    let mut out = std::io::sink();

    api.handle_position("position startpos moves e2e4", &mut out);
    let limits = api.parse_go_limits("go wtime 60000 btime 30000 winc 1000 binc 500");

    assert_eq!(limits.movetime_ms, Some(1100));
    assert_eq!(limits.wtime_ms, Some(60000));
    assert_eq!(limits.btime_ms, Some(30000));
}

#[test]
fn test_parse_go_limits_infinite_keeps_no_movetime() {
    let api = CodyApi::new(MaterialEvaluator::default());
    let limits = api.parse_go_limits("go infinite");

    assert!(limits.infinite);
    assert_eq!(limits.movetime_ms, None);
    assert_eq!(limits.depth, None);
}

#[test]
fn test_parse_go_limits_ponder_with_movetime_not_forced_infinite() {
    let api = CodyApi::new(MaterialEvaluator::default());
    let limits = api.parse_go_limits("go ponder movetime 50");

    assert!(limits.ponder);
    assert_eq!(limits.movetime_ms, Some(50));
    assert!(!limits.infinite);
}

#[test]
fn test_handle_setoption_verbose_toggles_global_flag() {
    let api = &mut CodyApi::new(MaterialEvaluator::default());

    api.handle_setoption("setoption name Verbose value true");
    assert!(VERBOSE.load(Ordering::Relaxed));

    api.handle_setoption("setoption name Verbose value false");
    assert!(!VERBOSE.load(Ordering::Relaxed));
}

#[test]
fn test_handle_setoption_ponder_toggles_runtime_option() {
    let api = &mut CodyApi::new(MaterialEvaluator::default());

    assert!(!api.ponder_enabled());
    api.handle_setoption("setoption name Ponder value true");
    assert!(api.ponder_enabled());

    api.handle_setoption("setoption name Ponder value false");
    assert!(!api.ponder_enabled());
}

#[test]
fn test_handle_setoption_syzygypath_is_accepted() {
    let api = &mut CodyApi::new(MaterialEvaluator::default());

    // Path may be invalid on CI/dev boxes; this test only validates parsing and
    // command handling stability.
    api.handle_setoption("setoption name SyzygyPath value C:\\tb");
}

#[allow(clippy::collapsible_if)]
#[test]
fn test_uci_position_moves_c3d5_state_consistency() {
    let api = &mut CodyApi::new(MaterialEvaluator::default());
    let mut out = std::io::sink();
    // Simulate: position fen ... moves c3d5
    let fen = "r2q1rk1/1p2bppp/p1npbn2/4p3/P3P3/1NN5/1PP1BPPP/R1BQ1R1K w - - 0 1";

    // Print all pseudo-legal moves for the FEN
    let pos = bitboard::position::Position::from_fen(fen);
    let mv = pos.parse_uci_move("c3d5").expect("Should parse c3d5");
    let mut new_pos = bitboard::position::Position::default();
    pos.apply_move_into(&mv, &mut new_pos);
    assert_eq!(
        new_pos.side_to_move,
        bitboard::piece::Color::Black,
        "Direct apply_move_into: should be Black to move after c3d5"
    );
    eprintln!("FEN after direct apply_move_into: {}", new_pos.to_fen());

    let cmd = format!("position fen {} moves c3d5", fen);
    api.handle_position(&cmd, &mut out);

    // After parsing, engine should think it is Black's turn
    eprintln!("FEN after UCI handler: {}", api.current_pos.to_fen());
    assert_eq!(
        api.current_pos.side_to_move,
        bitboard::piece::Color::Black,
        "Engine should have Black to move after c3d5"
    );

    // Helper to get piece at a square
    fn get_piece_at(
        pos: &bitboard::position::Position,
        sq: Square,
    ) -> Option<bitboard::piece::Piece> {
        let mask = bitboard::BitBoardMask::from_square(sq);
        for (piece, bb) in pos.pieces.iter() {
            if (bb & mask).is_nonempty() {
                return Some(piece);
            }
        }
        None
    }

    // C3 should be empty
    assert!(
        get_piece_at(&api.current_pos, Square::C3).is_none(),
        "C3 should be empty after c3d5"
    );

    // Black should have no legal moves from C3
    let black_moves = bitboard::movegen::generate_legal_moves(&api.current_pos);
    assert!(
        black_moves.iter().all(|m| m.from() != Square::C3),
        "No legal move should originate from C3 for Black"
    );

    // (Optional) Check if engine's best move is not from C3 (if search/TT is
    // accessible) let (bm, _score) = api.engine.search(&api.current_pos, 1,
    // Some(10), None); assert!(bm.from() != Square::C3, "Engine search
    // should not return a move from C3");
}
