// bitboard/src/tables/knight_attack.rs

use crate::BitBoardMask;
use crate::Square;

pub const KNIGHT_ATTACKS: [BitBoardMask; 64] = [
    BitBoardMask(0x0000_0000_0002_0400),
    BitBoardMask(0x0000_0000_0005_0800),
    BitBoardMask(0x0000_0000_000A_1100),
    BitBoardMask(0x0000_0000_0014_2200),
    BitBoardMask(0x0000_0000_0028_4400),
    BitBoardMask(0x0000_0000_0050_8800),
    BitBoardMask(0x0000_0000_00A0_1000),
    BitBoardMask(0x0000_0000_0040_2000),
    BitBoardMask(0x0000_0000_0204_0004),
    BitBoardMask(0x0000_0000_0508_0008),
    BitBoardMask(0x0000_0000_0A11_0011),
    BitBoardMask(0x0000_0000_1422_0022),
    BitBoardMask(0x0000_0000_2844_0044),
    BitBoardMask(0x0000_0000_5088_0088),
    BitBoardMask(0x0000_0000_A010_0010),
    BitBoardMask(0x0000_0000_4020_0020),
    BitBoardMask(0x0000_0002_0400_0402),
    BitBoardMask(0x0000_0005_0800_0805),
    BitBoardMask(0x0000_000A_1100_110A),
    BitBoardMask(0x0000_0014_2200_2214),
    BitBoardMask(0x0000_0028_4400_4428),
    BitBoardMask(0x0000_0050_8800_8850),
    BitBoardMask(0x0000_00A0_1000_10A0),
    BitBoardMask(0x0000_0040_2000_2040),
    BitBoardMask(0x0000_0204_0004_0200),
    BitBoardMask(0x0000_0508_0008_0500),
    BitBoardMask(0x0000_0A11_0011_0A00),
    BitBoardMask(0x0000_1422_0022_1400),
    BitBoardMask(0x0000_2844_0044_2800),
    BitBoardMask(0x0000_5088_0088_5000),
    BitBoardMask(0x0000_A010_0010_A000),
    BitBoardMask(0x0000_4020_0020_4000),
    BitBoardMask(0x0002_0400_0402_0000),
    BitBoardMask(0x0005_0800_0805_0000),
    BitBoardMask(0x000A_1100_110A_0000),
    BitBoardMask(0x0014_2200_2214_0000),
    BitBoardMask(0x0028_4400_4428_0000),
    BitBoardMask(0x0050_8800_8850_0000),
    BitBoardMask(0x00A0_1000_10A0_0000),
    BitBoardMask(0x0040_2000_2040_0000),
    BitBoardMask(0x0204_0004_0200_0000),
    BitBoardMask(0x0508_0008_0500_0000),
    BitBoardMask(0x0A11_0011_0A00_0000),
    BitBoardMask(0x1422_0022_1400_0000),
    BitBoardMask(0x2844_0044_2800_0000),
    BitBoardMask(0x5088_0088_5000_0000),
    BitBoardMask(0xA010_0010_A000_0000),
    BitBoardMask(0x4020_0020_4000_0000),
    BitBoardMask(0x0400_0402_0000_0000),
    BitBoardMask(0x0800_0805_0000_0000),
    BitBoardMask(0x1100_110A_0000_0000),
    BitBoardMask(0x2200_2214_0000_0000),
    BitBoardMask(0x4400_4428_0000_0000),
    BitBoardMask(0x8800_8850_0000_0000),
    BitBoardMask(0x1000_10A0_0000_0000),
    BitBoardMask(0x2000_2040_0000_0000),
    BitBoardMask(0x0004_0200_0000_0000),
    BitBoardMask(0x0008_0500_0000_0000),
    BitBoardMask(0x0011_0A00_0000_0000),
    BitBoardMask(0x0022_1400_0000_0000),
    BitBoardMask(0x0044_2800_0000_0000),
    BitBoardMask(0x0088_5000_0000_0000),
    BitBoardMask(0x0010_A000_0000_0000),
    BitBoardMask(0x0020_4000_0000_0000),
];

#[test]
fn test_knight_attacks_corner() {
    let attacks = KNIGHT_ATTACKS[Square::A1.index()];
    assert_eq!(attacks.0.count_ones(), 2);
}

#[test]
fn test_knight_attacks_center() {
    let attacks = KNIGHT_ATTACKS[Square::D4.index()];
    assert_eq!(attacks.0.count_ones(), 8);
}

#[test]
fn test_knight_attacks_edge() {
    let attacks = KNIGHT_ATTACKS[Square::A4.index()];
    assert_eq!(attacks.0.count_ones(), 4);
}

#[test]
fn test_knight_attacks_opposite_corner() {
    let attacks = KNIGHT_ATTACKS[Square::H8.index()];
    assert_eq!(attacks.0.count_ones(), 2);
}

