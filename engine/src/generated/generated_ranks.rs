// Auto-generated rank bitboards

pub const RANK_1: BitBoardMask = BitBoardMask(0x00000000000000FF);
pub const RANK_2: BitBoardMask = BitBoardMask(0x000000000000FF00);
pub const RANK_3: BitBoardMask = BitBoardMask(0x0000000000FF0000);
pub const RANK_4: BitBoardMask = BitBoardMask(0x00000000FF000000);
pub const RANK_5: BitBoardMask = BitBoardMask(0x000000FF00000000);
pub const RANK_6: BitBoardMask = BitBoardMask(0x0000FF0000000000);
pub const RANK_7: BitBoardMask = BitBoardMask(0x00FF000000000000);
pub const RANK_8: BitBoardMask = BitBoardMask(0xFF00000000000000);

pub const RANK_MASKS: [BitBoardMask; BOARD_SIZE] = [
    RANK_1,
    RANK_2,
    RANK_3,
    RANK_4,
    RANK_5,
    RANK_6,
    RANK_7,
    RANK_8,
];
