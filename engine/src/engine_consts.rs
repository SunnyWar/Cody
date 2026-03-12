// Engine-wide tunable constants for ELO and evaluation
// All values are in centipawns unless otherwise noted
// Tuning Range: [50, 150]
pub const MATERIAL_PAWN: i32 = 64;
// Tuning Range: [250, 350]
pub const MATERIAL_KNIGHT: i32 = 275;
// Tuning Range: [250, 350]
pub const MATERIAL_BISHOP: i32 = 284;
// Tuning Range: [400, 600]
pub const MATERIAL_ROOK: i32 = 477;
// Tuning Range: [800, 1200]
pub const MATERIAL_QUEEN: i32 = 1056;

// NOT TUNEABLE
pub const MATERIAL_KING: i32 = 0; // Not scored in material
// Tuning Range: [10, 50]
pub const BISHOP_PAIR_BONUS: i32 = 47;
// Tuning Range: [0, 30]
pub const DOUBLED_PAWN_PENALTY: i32 = 22;
// Tuning Range: [0, 30]
pub const ISOLATED_PAWN_PENALTY: i32 = 9;
// Tuning Range: [0, 100] per rank
pub const PASSED_PAWN_BONUS_BY_ADVANCE: [i32; 8] = [5, 5, 5, 13, 25, 65, 155, 5];
// Tuning Range: [0, 10]
pub const MOBILITY_WEIGHT: i32 = 3;
// Tuning Range: [0, 50]
pub const EXPOSED_KING_PENALTY: i32 = 23;
// Tuning Range: [0, 30]
pub const OPEN_FILE_NEAR_KING: i32 = 25;
// Tuning Range: [0, 40]
pub const KING_LACKING_ESCAPE_SQUARES: i32 = 11;
// Tuning Range: [0, 40]
pub const ROOK_ON_OPEN_FILE_BONUS: i32 = 31;
// Tuning Range: [0, 20]
pub const ROOK_ON_SEMIOPEN_FILE_BONUS: i32 = 12;
// Tuning Range: [0, 200] per rank
pub const PAWN_NEAR_PROMOTION: [i32; 8] = [5, 5, 5, 13, 25, 65, 155, 5];
