// bitboard/src/tables/knight_attack.rs

use crate::{BitBoardMask, Square};

pub const KNIGHT_ATTACKS: [BitBoardMask; 64] = [
    BitBoardMask(0x0000000000020400),
    BitBoardMask(0x0000000000050800),
    BitBoardMask(0x00000000000A1100),
    BitBoardMask(0x0000000000142200),
    BitBoardMask(0x0000000000284400),
    BitBoardMask(0x0000000000508800),
    BitBoardMask(0x0000000000A01000),
    BitBoardMask(0x0000000000402000),
    BitBoardMask(0x0000000002040004),
    BitBoardMask(0x0000000005080008),
    BitBoardMask(0x000000000A110011),
    BitBoardMask(0x0000000014220022),
    BitBoardMask(0x0000000028440044),
    BitBoardMask(0x0000000050880088),
    BitBoardMask(0x00000000A0100010),
    BitBoardMask(0x0000000040200020),
    BitBoardMask(0x0000000204000402),
    BitBoardMask(0x0000000508000805),
    BitBoardMask(0x0000000A1100110A),
    BitBoardMask(0x0000001422002214),
    BitBoardMask(0x0000002844004428),
    BitBoardMask(0x0000005088008850),
    BitBoardMask(0x000000A0100010A0),
    BitBoardMask(0x0000004020002040),
    BitBoardMask(0x0000020400040200),
    BitBoardMask(0x0000050800080500),
    BitBoardMask(0x00000A1100110A00),
    BitBoardMask(0x0000142200221400),
    BitBoardMask(0x0000284400442800),
    BitBoardMask(0x0000508800885000),
    BitBoardMask(0x0000A0100010A000),
    BitBoardMask(0x0000402000204000),
    BitBoardMask(0x0002040004020000),
    BitBoardMask(0x0005080008050000),
    BitBoardMask(0x000A1100110A0000),
    BitBoardMask(0x0014220022140000),
    BitBoardMask(0x0028440044280000),
    BitBoardMask(0x0050880088500000),
    BitBoardMask(0x00A0100010A00000),
    BitBoardMask(0x0040200020400000),
    BitBoardMask(0x0204000402000000),
    BitBoardMask(0x0508000805000000),
    BitBoardMask(0x0A1100110A000000),
    BitBoardMask(0x1422002214000000),
    BitBoardMask(0x2844004428000000),
    BitBoardMask(0x5088008850000000),
    BitBoardMask(0xA0100010A0000000),
    BitBoardMask(0x4020002040000000),
    BitBoardMask(0x0400040200000000),
    BitBoardMask(0x0800080500000000),
    BitBoardMask(0x1100110A00000000),
    BitBoardMask(0x2200221400000000),
    BitBoardMask(0x4400442800000000),
    BitBoardMask(0x8800885000000000),
    BitBoardMask(0x100010A000000000),
    BitBoardMask(0x2000204000000000),
    BitBoardMask(0x0004020000000000),
    BitBoardMask(0x0008050000000000),
    BitBoardMask(0x00110A0000000000),
    BitBoardMask(0x0022140000000000),
    BitBoardMask(0x0044280000000000),
    BitBoardMask(0x0088500000000000),
    BitBoardMask(0x0010A00000000000),
    BitBoardMask(0x0020400000000000),
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
