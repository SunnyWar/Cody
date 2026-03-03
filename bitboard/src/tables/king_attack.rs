// bitboard/src/tables/king_attack.rs
use crate::BitBoardMask;
use crate::Square;

pub const KING_ATTACKS: [BitBoardMask; 64] = [
    BitBoardMask(0x0000_0000_0000_0302),
    BitBoardMask(0x0000_0000_0000_0705),
    BitBoardMask(0x0000_0000_0000_0E0A),
    BitBoardMask(0x0000_0000_0000_1C14),
    BitBoardMask(0x0000_0000_0000_3828),
    BitBoardMask(0x0000_0000_0000_7050),
    BitBoardMask(0x0000_0000_0000_E0A0),
    BitBoardMask(0x0000_0000_0000_C040),
    BitBoardMask(0x0000_0000_0003_0203),
    BitBoardMask(0x0000_0000_0007_0507),
    BitBoardMask(0x0000_0000_000E_0A0E),
    BitBoardMask(0x0000_0000_001C_141C),
    BitBoardMask(0x0000_0000_0038_2838),
    BitBoardMask(0x0000_0000_0070_5070),
    BitBoardMask(0x0000_0000_00E0_A0E0),
    BitBoardMask(0x0000_0000_00C0_40C0),
    BitBoardMask(0x0000_0000_0302_0300),
    BitBoardMask(0x0000_0000_0705_0700),
    BitBoardMask(0x0000_0000_0E0A_0E00),
    BitBoardMask(0x0000_0000_1C14_1C00),
    BitBoardMask(0x0000_0000_3828_3800),
    BitBoardMask(0x0000_0000_7050_7000),
    BitBoardMask(0x0000_0000_E0A0_E000),
    BitBoardMask(0x0000_0000_C040_C000),
    BitBoardMask(0x0000_0003_0203_0000),
    BitBoardMask(0x0000_0007_0507_0000),
    BitBoardMask(0x0000_000E_0A0E_0000),
    BitBoardMask(0x0000_001C_141C_0000),
    BitBoardMask(0x0000_0038_2838_0000),
    BitBoardMask(0x0000_0070_5070_0000),
    BitBoardMask(0x0000_00E0_A0E0_0000),
    BitBoardMask(0x0000_00C0_40C0_0000),
    BitBoardMask(0x0000_0302_0300_0000),
    BitBoardMask(0x0000_0705_0700_0000),
    BitBoardMask(0x0000_0E0A_0E00_0000),
    BitBoardMask(0x0000_1C14_1C00_0000),
    BitBoardMask(0x0000_3828_3800_0000),
    BitBoardMask(0x0000_7050_7000_0000),
    BitBoardMask(0x0000_E0A0_E000_0000),
    BitBoardMask(0x0000_C040_C000_0000),
    BitBoardMask(0x0003_0203_0000_0000),
    BitBoardMask(0x0007_0507_0000_0000),
    BitBoardMask(0x000E_0A0E_0000_0000),
    BitBoardMask(0x001C_141C_0000_0000),
    BitBoardMask(0x0038_2838_0000_0000),
    BitBoardMask(0x0070_5070_0000_0000),
    BitBoardMask(0x00E0_A0E0_0000_0000),
    BitBoardMask(0x00C0_40C0_0000_0000),
    BitBoardMask(0x0302_0300_0000_0000),
    BitBoardMask(0x0705_0700_0000_0000),
    BitBoardMask(0x0E0A_0E00_0000_0000),
    BitBoardMask(0x1C14_1C00_0000_0000),
    BitBoardMask(0x3828_3800_0000_0000),
    BitBoardMask(0x7050_7000_0000_0000),
    BitBoardMask(0xE0A0_E000_0000_0000),
    BitBoardMask(0xC040_C000_0000_0000),
    BitBoardMask(0x0203_0000_0000_0000),
    BitBoardMask(0x0507_0000_0000_0000),
    BitBoardMask(0x0A0E_0000_0000_0000),
    BitBoardMask(0x141C_0000_0000_0000),
    BitBoardMask(0x2838_0000_0000_0000),
    BitBoardMask(0x5070_0000_0000_0000),
    BitBoardMask(0xA0E0_0000_0000_0000),
    BitBoardMask(0x40C0_0000_0000_0000),
];

#[allow(clippy::collapsible_if)]
#[test]
fn test_king_attacks_corner() {
    let attacks = KING_ATTACKS[Square::A1.index()];
    assert_eq!(attacks.0.count_ones(), 3);
}

#[test]
fn test_king_attacks_edge() {
    let attacks = KING_ATTACKS[Square::A4.index()];
    assert_eq!(attacks.0.count_ones(), 5);
}

#[test]
fn test_king_attacks_center() {
    let attacks = KING_ATTACKS[Square::D4.index()];
    assert_eq!(attacks.0.count_ones(), 8);
}

