// bitboard/src/tables/rank_masks.rs
use crate::BitBoardMask;
use crate::constants::BOARD_SIZE;

pub const RANK_1: BitBoardMask = BitBoardMask(0x0000_0000_0000_00FF);
pub const RANK_2: BitBoardMask = BitBoardMask(0x0000_0000_0000_FF00);
pub const RANK_3: BitBoardMask = BitBoardMask(0x0000_0000_00FF_0000);
pub const RANK_4: BitBoardMask = BitBoardMask(0x0000_0000_FF00_0000);
pub const RANK_5: BitBoardMask = BitBoardMask(0x0000_00FF_0000_0000);
pub const RANK_6: BitBoardMask = BitBoardMask(0x0000_FF00_0000_0000);
pub const RANK_7: BitBoardMask = BitBoardMask(0x00FF_0000_0000_0000);
pub const RANK_8: BitBoardMask = BitBoardMask(0xFF00_0000_0000_0000);

pub const RANK_MASKS: [BitBoardMask; BOARD_SIZE] = [
    RANK_1, RANK_2, RANK_3, RANK_4, RANK_5, RANK_6, RANK_7, RANK_8,
];

