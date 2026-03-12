// Engine-wide tunable constants for ELO and evaluation
// All values are in centipawns unless otherwise noted
// Tuning Range: [50, 150]
pub const MATERIAL_PAWN: i32 = 125;
// Tuning Range: [250, 350]
pub const MATERIAL_KNIGHT: i32 = 322;
// Tuning Range: [250, 350]
pub const MATERIAL_BISHOP: i32 = 271;
// Tuning Range: [400, 600]
pub const MATERIAL_ROOK: i32 = 448;
// Tuning Range: [800, 1200]
pub const MATERIAL_QUEEN: i32 = 964;

// NOT TUNEABLE
pub const MATERIAL_KING: i32 = 0; // Not scored in material
// Tuning Range: [10, 50]
pub const BISHOP_PAIR_BONUS: i32 = 22;
// Tuning Range: [0, 30]
pub const DOUBLED_PAWN_PENALTY: i32 = 0;
// Tuning Range: [0, 30]
pub const ISOLATED_PAWN_PENALTY: i32 = 3;
// Tuning Range: [0, 10]
pub const MOBILITY_WEIGHT: i32 = 8;
// Tuning Range: [0, 50]
pub const EXPOSED_KING_PENALTY: i32 = 13;
// Tuning Range: [0, 30]
pub const OPEN_FILE_NEAR_KING: i32 = 29;
// Tuning Range: [0, 40]
pub const KING_LACKING_ESCAPE_SQUARES: i32 = 36;
// Tuning Range: [0, 40]
pub const ROOK_ON_OPEN_FILE_BONUS: i32 = 10;
// Tuning Range: [0, 20]
pub const ROOK_ON_SEMIOPEN_FILE_BONUS: i32 = 5;

// Tuning Range: [0, 100] per rank
pub const PASSED_PAWN_BONUS_BY_ADVANCE_1: i32 = 5;
// Tuning Range: [0, 100] per rank
pub const PASSED_PAWN_BONUS_BY_ADVANCE_2: i32 = 5;
// Tuning Range: [0, 100] per rank
pub const PASSED_PAWN_BONUS_BY_ADVANCE_3: i32 = 5;
// Tuning Range: [0, 100] per rank
pub const PASSED_PAWN_BONUS_BY_ADVANCE_4: i32 = 13;
// Tuning Range: [0, 100] per rank
pub const PASSED_PAWN_BONUS_BY_ADVANCE_5: i32 = 25;
// Tuning Range: [0, 100] per rank
pub const PASSED_PAWN_BONUS_BY_ADVANCE_6: i32 = 65;
// Tuning Range: [0, 100] per rank
pub const PASSED_PAWN_BONUS_BY_ADVANCE_7: i32 = 155;
// Tuning Range: [0, 100] per rank
pub const PASSED_PAWN_BONUS_BY_ADVANCE_8: i32 = 5;

// NOT TUNEABLE
pub const PASSED_PAWN_BONUS_BY_ADVANCE: [i32; 8] = [
    PASSED_PAWN_BONUS_BY_ADVANCE_1,
    PASSED_PAWN_BONUS_BY_ADVANCE_2,
    PASSED_PAWN_BONUS_BY_ADVANCE_3,
    PASSED_PAWN_BONUS_BY_ADVANCE_4,
    PASSED_PAWN_BONUS_BY_ADVANCE_5,
    PASSED_PAWN_BONUS_BY_ADVANCE_6,
    PASSED_PAWN_BONUS_BY_ADVANCE_7,
    PASSED_PAWN_BONUS_BY_ADVANCE_8,
];

// Tuning Range: [0, 200] per rank
pub const PAWN_NEAR_PROMOTION_1: i32 = 5;
// Tuning Range: [0, 200] per rank
pub const PAWN_NEAR_PROMOTION_2: i32 = 5;
// Tuning Range: [0, 200] per rank
pub const PAWN_NEAR_PROMOTION_3: i32 = 5;
// Tuning Range: [0, 200] per rank
pub const PAWN_NEAR_PROMOTION_4: i32 = 13;
// Tuning Range: [0, 200] per rank
pub const PAWN_NEAR_PROMOTION_5: i32 = 25;
// Tuning Range: [0, 200] per rank
pub const PAWN_NEAR_PROMOTION_6: i32 = 65;
// Tuning Range: [0, 200] per rank
pub const PAWN_NEAR_PROMOTION_7: i32 = 155;
// Tuning Range: [0, 200] per rank
pub const PAWN_NEAR_PROMOTION_8: i32 = 5;

// NOT TUNEABLE
pub const PAWN_NEAR_PROMOTION: [i32; 8] = [
    PAWN_NEAR_PROMOTION_1,
    PAWN_NEAR_PROMOTION_2,
    PAWN_NEAR_PROMOTION_3,
    PAWN_NEAR_PROMOTION_4,
    PAWN_NEAR_PROMOTION_5,
    PAWN_NEAR_PROMOTION_6,
    PAWN_NEAR_PROMOTION_7,
    PAWN_NEAR_PROMOTION_8,
];
