// Piece-square tables for positional evaluation
// Square indexing: 0=a1, 7=h1, 8=a2, 15=h2, ..., 56=a8, 63=h8
// Tables are from White's perspective; Black pieces use flipped index (63 - sq)

// Pawn midgame table: encourage central control and pawn chains
pub const PAWN_SQUARE_TABLE: [i32; 64] = [
    // Rank 1 (unreachable for white pawns)
    0, 0, 0, 0, 0, 0, 0, 0, // Rank 2 - starting rank, prefer center pawns developed
    5, 10, 10, -20, -20, 10, 10, 5, // Rank 3 - slight development bonus
    5, -5, -10, 0, 0, -10, -5, 5, // Rank 4 - central pawns strong
    0, 0, 0, 20, 20, 0, 0, 0, // Rank 5 - good advancement
    5, 5, 10, 25, 25, 10, 5, 5, // Rank 6
    10, 10, 20, 30, 30, 20, 10, 10, // Rank 7 - strong advancement bonus
    50, 50, 50, 50, 50, 50, 50, 50, // Rank 8 (promotion, unreachable)
    0, 0, 0, 0, 0, 0, 0, 0,
];

// Knight midgame table: heavily reward center control and mobility
// Knights are strongest in the center, weakest on the rim
pub const KNIGHT_SQUARE_TABLE: [i32; 64] = [
    // Rank 1 - starting squares, want to develop
    -50, -40, -30, -30, -30, -30, -40, -50, // Rank 2 - undeveloped
    -40, -20, 0, 0, 0, 0, -20, -40, // Rank 3
    -30, 0, 10, 15, 15, 10, 0, -30, // Rank 4 - central dominance
    -30, 5, 15, 20, 20, 15, 5, -30, // Rank 5 - strong central outpost
    -30, 0, 15, 20, 20, 15, 0, -30, // Rank 6 - developed position
    -30, 5, 10, 15, 15, 10, 5, -30, // Rank 7
    -40, -20, 0, 5, 5, 0, -20, -40, // Rank 8 - bad squares (rim)
    -50, -40, -30, -30, -30, -30, -40, -50,
];

// Bishop midgame table: reward development and long diagonals
pub const BISHOP_SQUARE_TABLE: [i32; 64] = [
    // Rank 1 - starting squares, want to develop
    -20, -10, -10, -10, -10, -10, -10, -20, // Rank 2 - prefer developed
    -10, 5, 0, 0, 0, 0, 5, -10, // Rank 3 - good development squares
    -10, 10, 10, 10, 10, 10, 10, -10, // Rank 4
    -10, 0, 10, 10, 10, 10, 0, -10, // Rank 5
    -10, 5, 5, 10, 10, 5, 5, -10, // Rank 6
    -10, 0, 5, 10, 10, 5, 0, -10, // Rank 7
    -10, 0, 0, 0, 0, 0, 0, -10, // Rank 8
    -20, -10, -10, -10, -10, -10, -10, -20,
];

// Rook midgame table: prefer 7th rank and open files (approximated)
pub const ROOK_SQUARE_TABLE: [i32; 64] = [
    // Rank 1 - back rank, neutral
    0, 0, 0, 5, 5, 0, 0, 0, // Rank 2
    -5, 0, 0, 0, 0, 0, 0, -5, // Rank 3
    -5, 0, 0, 0, 0, 0, 0, -5, // Rank 4
    -5, 0, 0, 0, 0, 0, 0, -5, // Rank 5
    -5, 0, 0, 0, 0, 0, 0, -5, // Rank 6
    -5, 0, 0, 0, 0, 0, 0, -5, // Rank 7 - strong 7th rank rooks
    5, 10, 10, 10, 10, 10, 10, 5, // Rank 8
    0, 0, 0, 0, 0, 0, 0, 0,
];

// Queen midgame table: discourage early queen moves, central control
pub const QUEEN_SQUARE_TABLE: [i32; 64] = [
    // Rank 1 - starting square
    -20, -10, -10, -5, -5, -10, -10, -20, // Rank 2 - discourage early development
    -10, 0, 5, 0, 0, 0, 0, -10, // Rank 3
    -10, 5, 5, 5, 5, 5, 0, -10, // Rank 4
    0, 0, 5, 5, 5, 5, 0, -5, // Rank 5
    -5, 0, 5, 5, 5, 5, 0, -5, // Rank 6
    -10, 0, 5, 5, 5, 5, 0, -10, // Rank 7
    -10, 0, 0, 0, 0, 0, 0, -10, // Rank 8
    -20, -10, -10, -5, -5, -10, -10, -20,
];

// King midgame table: strong encouragement to castle, avoid center
// Heavy penalties for exposed king
pub const KING_MIDGAME_SQUARE_TABLE: [i32; 64] = [
    // Rank 1 - castled positions safe
    -10, -20, 30, -5, 0, -5, 30, -10, // Rank 2 - slightly better behind pawns
    -20, -30, -30, -40, -40, -30, -30, -20, // Rank 3
    -30, -40, -40, -50, -50, -40, -40, -30, // Rank 4
    -30, -40, -40, -50, -50, -40, -40, -30, // Rank 5
    -30, -40, -40, -50, -50, -40, -40, -30, // Rank 6
    -30, -40, -40, -50, -50, -40, -40, -30, // Rank 7
    -30, -40, -40, -50, -50, -40, -40, -30, // Rank 8
    -30, -40, -40, -50, -50, -40, -40, -30,
];

// Pawn endgame table: passed pawns very valuable, push for promotion
pub const PAWN_ENDGAME_TABLE: [i32; 64] = [
    // Rank 1
    0, 0, 0, 0, 0, 0, 0, 0, // Rank 2
    5, 5, 5, 5, 5, 5, 5, 5, // Rank 3
    5, 5, 5, 5, 5, 5, 5, 5, // Rank 4
    10, 10, 10, 10, 10, 10, 10, 10, // Rank 5
    20, 20, 20, 20, 20, 20, 20, 20, // Rank 6 - advanced passed pawns
    35, 35, 35, 35, 35, 35, 35, 35, // Rank 7 - very close to promotion
    50, 50, 50, 50, 50, 50, 50, 50, // Rank 8 (promotion)
    0, 0, 0, 0, 0, 0, 0, 0,
];

// Knight endgame table: still prefer center, but less critical
// Knights lose value in endgame (fewer pieces to coordinate with)
pub const KNIGHT_ENDGAME_TABLE: [i32; 64] = [
    // Rank 1
    -50, -40, -30, -30, -30, -30, -40, -50, // Rank 2
    -40, -20, 0, 5, 5, 0, -20, -40, // Rank 3
    -30, 5, 10, 15, 15, 10, 5, -30, // Rank 4
    -30, 0, 15, 20, 20, 15, 0, -30, // Rank 5
    -30, 5, 15, 20, 20, 15, 5, -30, // Rank 6
    -30, 0, 10, 15, 15, 10, 0, -30, // Rank 7
    -40, -20, 0, 5, 5, 0, -20, -40, // Rank 8
    -50, -40, -30, -30, -30, -30, -40, -50,
];

// Bishop endgame table: bishops gain value in endgame (long range)
pub const BISHOP_ENDGAME_TABLE: [i32; 64] = [
    // Rank 1
    -20, -10, -10, -10, -10, -10, -10, -20, // Rank 2
    -10, 5, 0, 0, 0, 0, 5, -10, // Rank 3
    -10, 10, 10, 10, 10, 10, 10, -10, // Rank 4
    -10, 0, 10, 10, 10, 10, 0, -10, // Rank 5
    -10, 5, 5, 10, 10, 5, 5, -10, // Rank 6
    -10, 0, 5, 10, 10, 5, 0, -10, // Rank 7
    -10, 0, 0, 0, 0, 0, 0, -10, // Rank 8
    -20, -10, -10, -10, -10, -10, -10, -20,
];

// Rook endgame table: active rooks, 7th rank still valuable
pub const ROOK_ENDGAME_TABLE: [i32; 64] = [
    // Rank 1
    0, 0, 0, 0, 0, 0, 0, 0, // Rank 2
    0, 0, 0, 0, 0, 0, 0, 0, // Rank 3
    0, 0, 0, 0, 0, 0, 0, 0, // Rank 4
    0, 0, 0, 0, 0, 0, 0, 0, // Rank 5
    0, 0, 0, 0, 0, 0, 0, 0, // Rank 6
    5, 5, 5, 5, 5, 5, 5, 5, // Rank 7 - still powerful
    10, 10, 10, 10, 10, 10, 10, 10, // Rank 8
    0, 0, 0, 0, 0, 0, 0, 0,
];

// Queen endgame table: centralized and active
pub const QUEEN_ENDGAME_TABLE: [i32; 64] = [
    // Rank 1
    -20, -10, -10, -5, -5, -10, -10, -20, // Rank 2
    -10, 0, 0, 5, 5, 0, 0, -10, // Rank 3
    -10, 0, 5, 5, 5, 5, 0, -10, // Rank 4
    -5, 0, 5, 5, 5, 5, 0, -5, // Rank 5
    -5, 0, 5, 5, 5, 5, 0, -5, // Rank 6
    -10, 5, 5, 5, 5, 5, 5, -10, // Rank 7
    -10, 0, 5, 5, 5, 5, 0, -10, // Rank 8
    -20, -10, -10, -5, -5, -10, -10, -20,
];

// King endgame table: STRONGLY encourage centralization
// King becomes a fighting piece in the endgame
pub const KING_ENDGAME_TABLE: [i32; 64] = [
    // Rank 1 - away from action
    -50, -40, -30, -20, -20, -30, -40, -50, // Rank 2
    -30, -20, -10, 0, 0, -10, -20, -30, // Rank 3
    -30, -10, 20, 30, 30, 20, -10, -30, // Rank 4 - central squares very strong
    -30, 0, 30, 40, 40, 30, 0, -30, // Rank 5 - getting closer to center
    -30, 0, 30, 40, 40, 30, 0, -30, // Rank 6
    -30, -10, 20, 30, 30, 20, -10, -30, // Rank 7
    -30, -20, -10, 0, 0, -10, -20, -30, // Rank 8 - far from center
    -50, -40, -30, -20, -20, -30, -40, -50,
];

pub const PHASE_WEIGHTS: [i32; 6] = [
    0, // Pawn
    1, // Knight
    1, // Bishop
    2, // Rook
    4, // Queen
    0, // King
];
pub const MAX_PHASE: i32 = 24; // 4 queens + 4 rooks + 4 minor pieces
