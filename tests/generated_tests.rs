use cody::{Engine, MaterialEvaluator, SimpleMoveGen, TEST_CASES};

#[test]
fn initial_position() {
let case = &TEST_CASES[0];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn kiwipete() {
let case = &TEST_CASES[1];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn position_4() {
let case = &TEST_CASES[2];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn double_rook_pressure() {
let case = &TEST_CASES[3];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn knight_maze() {
let case = &TEST_CASES[4];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn pinned_and_pushing() {
let case = &TEST_CASES[5];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn queen_infiltration() {
let case = &TEST_CASES[6];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn knight_fork_pressure() {
let case = &TEST_CASES[7];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn central_blockade() {
let case = &TEST_CASES[8];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn rook_battery_strike() {
let case = &TEST_CASES[9];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn minor_piece_gridlock() {
let case = &TEST_CASES[10];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn minor_piece_congestion() {
let case = &TEST_CASES[11];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn queen_side_pressure() {
let case = &TEST_CASES[12];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn pawn_chain_breaker() {
let case = &TEST_CASES[13];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn rook_activity_test() {
let case = &TEST_CASES[14];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn central_tension() {
let case = &TEST_CASES[15];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn pawn_labyrinth() {
let case = &TEST_CASES[16];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn pawn_wall_endgame() {
let case = &TEST_CASES[17];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn rook_escape() {
let case = &TEST_CASES[18];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn queen_vs_pawns() {
let case = &TEST_CASES[19];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn pawn_standoff() {
let case = &TEST_CASES[20];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn pawn_push_endgame() {
let case = &TEST_CASES[21];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn pawn_chain_vs_rook() {
let case = &TEST_CASES[22];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn locked_pawn_battle() {
let case = &TEST_CASES[23];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn rook_vs_pawn_race() {
let case = &TEST_CASES[24];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn knight_blockade() {
let case = &TEST_CASES[25];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn bishop_vs_pawn_wall() {
let case = &TEST_CASES[26];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn rook_lift_pressure() {
let case = &TEST_CASES[27];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn bishop_endgame_grind() {
let case = &TEST_CASES[28];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn queen_vs_rooks() {
let case = &TEST_CASES[29];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn double_rook_threat() {
let case = &TEST_CASES[30];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn castling_complexity() {
let case = &TEST_CASES[31];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn minor_piece_standoff() {
let case = &TEST_CASES[32];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn knight_web() {
let case = &TEST_CASES[33];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn knight_vs_pawn_wall() {
let case = &TEST_CASES[34];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn knight_endgame_dance() {
let case = &TEST_CASES[35];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn triple_bishop_coordination() {
let case = &TEST_CASES[36];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn passed_pawn_sprint() {
let case = &TEST_CASES[37];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn rook_cover_pawn_thrust() {
let case = &TEST_CASES[38];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn knight_fork_endgame() {
let case = &TEST_CASES[39];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn queen_vs_two_rooks_trap() {
let case = &TEST_CASES[40];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn queen_threat_promotion() {
let case = &TEST_CASES[41];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn rook_lift_attack() {
let case = &TEST_CASES[42];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn pawn_vs_king_endgame() {
let case = &TEST_CASES[43];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn promotion_race() {
let case = &TEST_CASES[44];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn fairy_castling_test() {
let case = &TEST_CASES[45];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

#[test]
fn grand_chess_opening() {
let case = &TEST_CASES[46];
let mut engine = Engine::new(1024, SimpleMoveGen, MaterialEvaluator);
let (_, score) = engine.search(&case.position(), 4);
assert_eq!(
score,
case.expected_score,
"Test: {}\nFEN: {}\nExpected Score: {}\nActual Score: {}",
case.name,
case.fen,
case.expected_score,
score
);
}

