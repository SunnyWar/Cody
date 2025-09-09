use crate::core::position::Position;
use once_cell::sync::Lazy;

pub struct TestCase {
    pub name: &'static str,
    pub fen: &'static str,
    pub expected_score: i32,
    pub expected_move: &'static str,
}

impl TestCase {
    pub fn position(&self) -> Position {
        Position::from_fen(self.fen)
    }
}
pub static TEST_CASES: Lazy<Vec<TestCase>> = Lazy::new(|| {
    vec![
        TestCase {
            name: "Initial_Position",
            fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            expected_score: 0,
            expected_move: "e2e4",
        },
        TestCase {
            name: "Kiwipete",
            fen: "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 10",
            expected_score: 0,
            expected_move: "e5f6",
        },
        TestCase {
            name: "Position_4",
            fen: "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 11",
            expected_score: 0,
            expected_move: "a2a4",
        },
        TestCase {
            name: "Double_Rook_Pressure",
            fen: "4rrk1/pp1n3p/3q2pQ/2p1pb2/2PP4/2P3N1/P2B2PP/4RRK1 b - - 7 19",
            expected_score: 0,
            expected_move: "f5g4",
        },
        TestCase {
            name: "Knight_Maze",
            fen: "rq3rk1/ppp2ppp/1bnpN3/3N2B1/4P3/7P/PPPQ1PP1/2KR3R b - - 0 14",
            expected_score: 320,
            expected_move: "c6d4",
        },
        TestCase {
            name: "Pinned_and_Pushing",
            fen: "r1bq1r1k/1pp1n1pp/1p1p4/4p2Q/4PpP1/1BNP4/PPP2P1P/3R1RK1 b - g3 0 14",
            expected_score: 0,
            expected_move: "g7g6",
        },
        TestCase {
            name: "queen_infiltration",
            fen: "r3r1k1/2p2ppp/p1p1bn2/8/1q2P3/2NPQN2/PPP3PP/R4RK1 b - - 2 15",
            expected_score: 90,
            expected_move: "b4b2",
        },
        TestCase {
            name: "knight_fork_pressure",
            fen: "r1bbk1nr/pp3p1p/2n5/1N4p1/2Np1B2/8/PPP2PPP/2KR1B1R w kq - 0 13",
            expected_score: 0,
            expected_move: "d4c6",
        },
        TestCase {
            name: "central_blockade",
            fen: "r1bq1rk1/ppp1nppp/4n3/3p3Q/3P4/1BP1B3/PP1N2PP/R4RK1 w - - 1 16",
            expected_score: -90,
            expected_move: "f1f3",
        },
        TestCase {
            name: "rook_battery_strike",
            fen: "4r1k1/r1q2ppp/ppp2n2/4P3/5Rb1/1N1BQ3/PPP3PP/R5K1 w - - 1 17",
            expected_score: 0,
            expected_move: "e5f6",
        },
        TestCase {
            name: "minor_piece_gridlock",
            fen: "2rqkb1r/ppp2p2/2npb1p1/1N1Nn2p/2P1PP2/8/PP2B1PP/R1BQK2R b KQ - 0 11",
            expected_score: 0,
            expected_move: "d6c5",
        },
        TestCase {
            name: "minor_piece_congestion",
            fen: "r1bq1r1k/b1p1npp1/p2p3p/1p6/3PP3/1B2NN2/PP3PPP/R2Q1RK1 w - - 1 16",
            expected_score: -10,
            expected_move: "d1d2",
        },
        TestCase {
            name: "queen_side_pressure",
            fen: "3r1rk1/p5pp/bpp1pp2/8/q1PP1P2/b3P3/P2NQRPP/1R2B1K1 b - - 6 22",
            expected_score: -10,
            expected_move: "a3c1",
        },
        TestCase {
            name: "pawn_chain_breaker",
            fen: "r1q2rk1/2p1bppp/2Pp4/p6b/Q1PNp3/4B3/PP1R1PPP/2K4R w - - 2 18",
            expected_score: -10,
            expected_move: "d4b5",
        },
        TestCase {
            name: "rook_activity_test",
            fen: "4k2r/1pb2ppp/1p2p3/1R1p4/3P4/2r1PN2/P4PPP/1R4K1 b - - 3 22",
            expected_score: -110,
            expected_move: "c3c2",
        },
        TestCase {
            name: "central_tension",
            fen: "3q2k1/pb3p1p/4pbp1/2r5/PpN2N2/1P2P2P/5PP1/Q2R2K1 b - - 4 26",
            expected_score: -20,
            expected_move: "d8d5",
        },
        TestCase {
            name: "pawn_labyrinth",
            fen: "6k1/6p1/6Pp/ppp5/3pn2P/1P3K2/1PP2P2/3N4 b - - 0 1",
            expected_score: 0,
            expected_move: "d4c3",
        },
        TestCase {
            name: "pawn_wall_endgame",
            fen: "3b4/5kp1/1p1p1p1p/pP1PpP1P/P1P1P3/3KN3/8/8 w - - 0 1",
            expected_score: -10,
            expected_move: "d3c2",
        },
        TestCase {
            name: "rook_escape",
            fen: "2K5/p7/7P/5pR1/8/5k2/r7/8 w - - 4 3",
            expected_score: -100,
            expected_move: "g5g6",
        },
        TestCase {
            name: "queen_vs_pawns",
            fen: "8/6pk/1p6/8/PP3p1p/5P2/4KP1q/3Q4 w - - 0 1",
            expected_score: -100,
            expected_move: "d1d3",
        },
        TestCase {
            name: "pawn_standoff",
            fen: "8/2p5/8/2kPKp1p/2p4P/2P5/3P4/8 w - - 0 1",
            expected_score: 0,
            expected_move: "d5d6",
        },
        TestCase {
            name: "pawn_push_endgame",
            fen: "8/1p3pp1/7p/5P1P/2k3P1/8/2K2P2/8 w - - 0 1",
            expected_score: 0,
            expected_move: "f5f6",
        },
        TestCase {
            name: "pawn_chain_vs_rook",
            fen: "8/pp2r1k1/2p1p3/3pP2p/1P1P1P1P/P5KR/8/8 w - - 0 1",
            expected_score: 0,
            expected_move: "f4f5",
        },
        TestCase {
            name: "locked_pawn_battle",
            fen: "8/3p4/p1bk3p/Pp6/1Kp1PpPp/2P2P1P/2P5/5B2 b - - 0 1",
            expected_score: 0,
            expected_move: "d6e5",
        },
        TestCase {
            name: "rook_vs_pawn_race",
            fen: "5k2/7R/4P2p/5K2/p1r2P1p/8/8/8 b - - 0 1",
            expected_score: -100,
            expected_move: "c4c2",
        },
        TestCase {
            name: "knight_blockade",
            fen: "6k1/6p1/P6p/r1N5/5p2/7P/1b3PP1/4R1K1 w - - 0 1",
            expected_score: 90,
            expected_move: "a6a7",
        },
        TestCase {
            name: "bishop_vs_pawn_wall",
            fen: "1r3k2/4q3/2Pp3b/3Bp3/2Q2p2/1p1P2P1/1P2KP2/3N4 w - - 0 1",
            expected_score: -80,
            expected_move: "c4c3",
        },
        TestCase {
            name: "rook_lift_pressure",
            fen: "6k1/4pp1p/3p2p1/P1pPb3/R7/1r2P1PP/3B1P2/6K1 w - - 0 1",
            expected_score: 0,
            expected_move: "a5a6",
        },
        TestCase {
            name: "bishop_endgame_grind",
            fen: "8/3p3B/5p2/5P2/p7/PP5b/k7/6K1 w - - 0 1",
            expected_score: 0,
            expected_move: "h7g6",
        },
        TestCase {
            name: "queen_vs_rooks",
            fen: "5rk1/q6p/2p3bR/1pPp1rP1/1P1Pp3/P3B1Q1/1K3P2/R7 w - - 93 90",
            expected_score: 0,
            expected_move: "g3h4",
        },
        TestCase {
            name: "double_rook_threat",
            fen: "4rrk1/1p1nq3/p7/2p1P1pp/3P2bp/3Q1Bn1/PPPB4/1K2R1NR w - - 40 21",
            expected_score: -90,
            expected_move: "d3g6",
        },
        TestCase {
            name: "castling_complexity",
            fen: "r3k2r/3nnpbp/q2pp1p1/p7/Pp1PPPP1/4BNN1/1P5P/R2Q1RK1 w kq - 0 16",
            expected_score: 0,
            expected_move: "f1e1",
        },
        TestCase {
            name: "minor_piece_standoff",
            fen: "3Qb1k1/1r2ppb1/pN1n2q1/Pp1Pp1Pr/4P2p/4BP2/4B1R1/1R5K b - - 11 40",
            expected_score: -100,
            expected_move: "g7h6",
        },
        TestCase {
            name: "knight_web",
            fen: "4k3/3q1r2/1N2r1b1/3ppN2/2nPP3/1B1R2n1/2R1Q3/3K4 w - - 5 1",
            expected_score: 0,
            expected_move: "d1c1",
        },
        TestCase {
            name: "knight_vs_pawn_wall",
            fen: "8/8/8/8/5kp1/P7/8/1K1N4 w - - 0 1",
            expected_score: 320,
            expected_move: "a3a4",
        },
        TestCase {
            name: "knight_endgame_dance",
            fen: "8/8/8/5N2/8/p7/8/2NK3k w - - 0 1",
            expected_score: 540,
            expected_move: "e5f3",
        },
        TestCase {
            name: "triple_bishop_coordination",
            fen: "8/3k4/8/8/8/4B3/4KB2/2B5 w - - 0 1",
            expected_score: 990,
            expected_move: "e2f3",
        },
        TestCase {
            name: "passed_pawn_sprint",
            fen: "8/8/1P6/5pr1/8/4R3/7k/2K5 w - - 0 1",
            expected_score: 0,
            expected_move: "b6b7",
        },
        TestCase {
            name: "rook_cover_pawn_thrust",
            fen: "8/2p4P/8/kr6/6R1/8/8/1K6 w - - 0 1",
            expected_score: 0,
            expected_move: "g4g5",
        },
        TestCase {
            name: "knight_fork_endgame",
            fen: "8/8/3P3k/8/1p6/8/1P6/1K3n2 b - - 0 1",
            expected_score: -220,
            expected_move: "f1e3",
        },
        TestCase {
            name: "queen_vs_two_rooks_trap",
            fen: "8/R7/2q5/8/6k1/8/1P5p/K6R w - - 0 124",
            expected_score: 100,
            expected_move: "a1a4",
        },
        TestCase {
            name: "queen_threat_promotion",
            fen: "6k1/3b3r/1p1p4/p1n2p2/1PPNpP1q/P3Q1p1/1R1RB1P1/5K2 b - - 0 1",
            expected_score: 400,
            expected_move: "h2h1",
        },
        TestCase {
            name: "rook_lift_attack",
            fen: "r2r1n2/pp2bk2/2p1p2p/3q4/3PN1QP/2P3R1/P4PP1/5RK1 w - - 0 1",
            expected_score: -230,
            expected_move: "g3g7",
        },
        TestCase {
            name: "pawn_vs_king_endgame",
            fen: "8/8/8/8/8/6k1/6p1/6K1 w - - 0 1",
            expected_score: -100,
            expected_move: "g1h1",
        },
        TestCase {
            name: "promotion_race",
            fen: "7k/7P/6K1/8/3B4/8/8/8 b - - 0 1",
            expected_score: 430,
            expected_move: "h7h6",
        },
        TestCase {
            name: "fairy_castling_test",
            fen: "bb1n1rkr/ppp1Q1pp/3n1p2/3p4/3P4/6Pq/PPP1PP1P/BB1NNRKR w HFhf - 0 5",
            expected_score: 100,
            expected_move: "e7d6",
        },
        TestCase {
            name: "grand_chess_opening",
            fen: "nqbnrkrb/pppppppp/8/8/8/8/PPPPPPPP/NQBNRKRB w GEge - 0 1",
            expected_score: 0,
            expected_move: "d2d4",
        },
    ]
});
