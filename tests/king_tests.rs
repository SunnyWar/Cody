use cody::{
    core::{bitboard::king_attacks, bitboardmask::BitBoardMask, square::Square},
    generated::KING_ATTACKS,
};

#[test]
fn test_king_attack_table_correctness() {
    for square in Square::all_array() {
        let expected = king_attacks(square);
        let actual = BitBoardMask(KING_ATTACKS[square.index()]);

        assert_eq!(
            actual.0, expected.0,
            "Mismatch at square {:?}: expected {:016X}, got {:016X}",
            square, expected.0, actual.0
        );
    }
}
