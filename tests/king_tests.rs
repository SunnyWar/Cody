use cody::{
    core::{bitboard::king_attacks, square::Square},
    generated::KING_ATTACKS,
};

#[test]
fn test_king_attack_table_correctness() {
    for square in Square::all_array() {
        let expected = king_attacks(square);
        let actual = KING_ATTACKS[square.index()];

        assert_eq!(
            actual.0, expected.0,
            "Mismatch at square {:?}: expected {:016X}, got {:016X}",
            square, expected.0, actual.0
        );
    }
}

#[test]
fn test_king_attack_table_specific_positions() {
    use Square::*;

    let test_cases = [
        (A1, 0x0000000000000302),
        (D4, 0x0000001C141C0000),
        (H1, 0x000000000000C040),
        (A8, 0x0203000000000000),
        (H8, 0x40C0000000000000),
        (E5, 0x0000382838000000),
    ];

    for (square, expected_mask) in test_cases {
        let actual = KING_ATTACKS[square.index()];
        assert_eq!(
            actual.0, expected_mask,
            "Incorrect king attack mask for {:?}: expected {:016X}, got {:016X}",
            square, expected_mask, actual.0
        );
    }
}
