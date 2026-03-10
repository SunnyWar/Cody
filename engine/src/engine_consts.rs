// Engine-wide tunable constants for ELO and evaluation
// All values are in centipawns unless otherwise noted

pub const MATERIAL_PAWN: i32 = 100;
pub const MATERIAL_KNIGHT: i32 = 320;
pub const MATERIAL_BISHOP: i32 = 330;
pub const MATERIAL_ROOK: i32 = 500;
pub const MATERIAL_QUEEN: i32 = 900;
pub const MATERIAL_KING: i32 = 0; // Not scored in material

pub const BISHOP_PAIR_BONUS: i32 = 30;
pub const DOUBLED_PAWN_PENALTY: i32 = 12;
pub const ISOLATED_PAWN_PENALTY: i32 = 10;
pub const PASSED_PAWN_BONUS_BY_ADVANCE: [i32; 8] = [0, 5, 10, 18, 28, 42, 60, 0];

pub const MOBILITY_WEIGHT: i32 = 4;

pub const EXPOSED_KING_PENALTY: i32 = 25;
pub const OPEN_FILE_NEAR_KING: i32 = 15;
pub const KING_LACKING_ESCAPE_SQUARES: i32 = 20;

pub const ROOK_ON_OPEN_FILE_BONUS: i32 = 20;
pub const ROOK_ON_SEMIOPEN_FILE_BONUS: i32 = 10;

pub const PAWN_NEAR_PROMOTION: [i32; 8] = [5, 5, 5, 13, 25, 65, 155, 5];
