// Engine-wide tunable constants for ELO and evaluation
// All values are in centipawns unless otherwise noted
// Tuning Range: [50, 150]
pub const MATERIAL_PAWN: i32 = 117;
// Tuning Range: [250, 350]
pub const MATERIAL_KNIGHT: i32 = 285;
// Tuning Range: [250, 350]
pub const MATERIAL_BISHOP: i32 = 309;
// Tuning Range: [400, 600]
pub const MATERIAL_ROOK: i32 = 528;
// Tuning Range: [800, 1200]
pub const MATERIAL_QUEEN: i32 = 858;

// NOT TUNEABLE
pub const MATERIAL_KING: i32 = 0; // Not scored in material
// Tuning Range: [10, 50]
pub const BISHOP_PAIR_BONUS: i32 = 14;
// Tuning Range: [0, 30]
pub const DOUBLED_PAWN_PENALTY: i32 = 29;
// Tuning Range: [0, 30]
pub const ISOLATED_PAWN_PENALTY: i32 = 14;
// Tuning Range: [0, 100] per rank
pub const PASSED_PAWN_BONUS_BY_ADVANCE: [i32; 8] = [5, 5, 5, 13, 25, 65, 155, 5];
// Tuning Range: [0, 10]
pub const MOBILITY_WEIGHT: i32 = 1;
// Tuning Range: [0, 50]
pub const EXPOSED_KING_PENALTY: i32 = 16;
// Tuning Range: [0, 30]
pub const OPEN_FILE_NEAR_KING: i32 = 26;
// Tuning Range: [0, 40]
pub const KING_LACKING_ESCAPE_SQUARES: i32 = 12;
// Tuning Range: [0, 40]
pub const ROOK_ON_OPEN_FILE_BONUS: i32 = 32;
// Tuning Range: [0, 20]
pub const ROOK_ON_SEMIOPEN_FILE_BONUS: i32 = 17;
// Tuning Range: [0, 200] per rank
pub const PAWN_NEAR_PROMOTION: [i32; 8] = [5, 5, 5, 13, 25, 65, 155, 5];
