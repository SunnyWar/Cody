// bitboard/src/tables/file_attack.rs
use crate::BitBoardMask;
use crate::constants::BOARD_SIZE;

pub const FILE_A: BitBoardMask = BitBoardMask(0x0101010101010101);
pub const FILE_B: BitBoardMask = BitBoardMask(0x0202020202020202);
pub const FILE_C: BitBoardMask = BitBoardMask(0x0404040404040404);
pub const FILE_D: BitBoardMask = BitBoardMask(0x0808080808080808);
pub const FILE_E: BitBoardMask = BitBoardMask(0x1010101010101010);
pub const FILE_F: BitBoardMask = BitBoardMask(0x2020202020202020);
pub const FILE_G: BitBoardMask = BitBoardMask(0x4040404040404040);
pub const FILE_H: BitBoardMask = BitBoardMask(0x8080808080808080);

pub const FILE_MASKS: [BitBoardMask; BOARD_SIZE] = [
    FILE_A, FILE_B, FILE_C, FILE_D, FILE_E, FILE_F, FILE_G, FILE_H,
];

// Masks excluding specific files
pub const NOT_FILE_A: BitBoardMask = BitBoardMask(0xFEFEFEFEFEFEFEFE);
pub const NOT_FILE_AB: BitBoardMask = BitBoardMask(0xFCFCFCFCFCFCFCFC);
pub const NOT_FILE_H: BitBoardMask = BitBoardMask(0x7F7F7F7F7F7F7F7F);
pub const NOT_FILE_GH: BitBoardMask = BitBoardMask(0x3F3F3F3F3F3F3F3F);
