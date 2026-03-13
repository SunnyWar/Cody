// Engine-wide tunable constants for ELO and evaluation
// All values are in centipawns unless otherwise noted
// Tuning Range: [50, 150]
pub const MATERIAL_PAWN: i32 = 82;
// Tuning Range: [250, 350]
pub const MATERIAL_KNIGHT: i32 = 282;
// Tuning Range: [250, 350]
pub const MATERIAL_BISHOP: i32 = 319;
// Tuning Range: [400, 600]
pub const MATERIAL_ROOK: i32 = 500;
// Tuning Range: [800, 1200]
pub const MATERIAL_QUEEN: i32 = 946;

// NOT TUNEABLE
pub const MATERIAL_KING: i32 = 0; // Not scored in material
// Tuning Range: [10, 50]
pub const BISHOP_PAIR_BONUS: i32 = 15;
// Tuning Range: [0, 30]
pub const DOUBLED_PAWN_PENALTY: i32 = 21;
// Tuning Range: [0, 30]
pub const ISOLATED_PAWN_PENALTY: i32 = 24;
// Tuning Range: [0, 10]
pub const MOBILITY_WEIGHT: i32 = 6;
// Tuning Range: [0, 50]
pub const EXPOSED_KING_PENALTY: i32 = 14;
// Tuning Range: [0, 30]
pub const OPEN_FILE_NEAR_KING: i32 = 18;
// Tuning Range: [0, 40]
pub const KING_LACKING_ESCAPE_SQUARES: i32 = 8;
// Tuning Range: [0, 40]
pub const ROOK_ON_OPEN_FILE_BONUS: i32 = 7;
// Tuning Range: [0, 20]
pub const ROOK_ON_SEMIOPEN_FILE_BONUS: i32 = 9;

// Tuning Range: [0, 100] per rank
pub const PPBBA_1: i32 = 5;
// Tuning Range: [0, 100] per rank
pub const PPBBA_2: i32 = 5;
// Tuning Range: [0, 100] per rank
pub const PPBBA_3: i32 = 5;
// Tuning Range: [0, 100] per rank
pub const PPBBA_4: i32 = 13;
// Tuning Range: [0, 100] per rank
pub const PPBBA_5: i32 = 25;
// Tuning Range: [0, 100] per rank
pub const PPBBA_6: i32 = 65;
// Tuning Range: [0, 100] per rank
pub const PPBBA_7: i32 = 155;
// Tuning Range: [0, 100] per rank
pub const PPBBA_8: i32 = 5;

// NOT TUNEABLE
pub const PASSED_PAWN_BONUS_BY_ADVANCE: [i32; 8] = [
    PPBBA_1, PPBBA_2, PPBBA_3, PPBBA_4, PPBBA_5, PPBBA_6, PPBBA_7, PPBBA_8,
];

// Tuning Range: [0, 200] per rank
pub const PNP_1: i32 = 5;
// Tuning Range: [0, 200] per rank
pub const PNP_2: i32 = 5;
// Tuning Range: [0, 200] per rank
pub const PNP_3: i32 = 5;
// Tuning Range: [0, 200] per rank
pub const PNP_4: i32 = 13;
// Tuning Range: [0, 200] per rank
pub const PNP_5: i32 = 25;
// Tuning Range: [0, 200] per rank
pub const PNP_6: i32 = 65;
// Tuning Range: [0, 200] per rank
pub const PNP_7: i32 = 155;
// Tuning Range: [0, 200] per rank
pub const PNP_8: i32 = 5;

// NOT TUNEABLE
pub const PAWN_NEAR_PROMOTION: [i32; 8] = [PNP_1, PNP_2, PNP_3, PNP_4, PNP_5, PNP_6, PNP_7, PNP_8];
