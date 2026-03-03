// bitboard/src/tables/file_attack.rs
use crate::BitBoardMask;
use crate::constants::BOARD_SIZE;

pub const FILE_A: BitBoardMask = BitBoardMask(0x0101_0101_0101_0101);
pub const FILE_B: BitBoardMask = BitBoardMask(0x0202_0202_0202_0202);
pub const FILE_C: BitBoardMask = BitBoardMask(0x0404_0404_0404_0404);
pub const FILE_D: BitBoardMask = BitBoardMask(0x0808_0808_0808_0808);
pub const FILE_E: BitBoardMask = BitBoardMask(0x1010_1010_1010_1010);
pub const FILE_F: BitBoardMask = BitBoardMask(0x2020_2020_2020_2020);
pub const FILE_G: BitBoardMask = BitBoardMask(0x4040_4040_4040_4040);
pub const FILE_H: BitBoardMask = BitBoardMask(0x8080_8080_8080_8080);

pub const FILE_MASKS: [BitBoardMask; BOARD_SIZE] = [
    FILE_A, FILE_B, FILE_C, FILE_D, FILE_E, FILE_F, FILE_G, FILE_H,
];

// Masks excluding specific files
pub const NOT_FILE_A: BitBoardMask = BitBoardMask(0xFEFE_FEFE_FEFE_FEFE);
pub const NOT_FILE_AB: BitBoardMask = BitBoardMask(0xFCFC_FCFC_FCFC_FCFC);
pub const NOT_FILE_H: BitBoardMask = BitBoardMask(0x7F7F_7F7F_7F7F_7F7F);
pub const NOT_FILE_GH: BitBoardMask = BitBoardMask(0x3F3F_3F3F_3F3F_3F3F);

