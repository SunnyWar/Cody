// tests/ranks_tests.rs

use cody::generated::{RANK_1, RANK_8, RANK_MASKS};

#[test]
fn test_rank_1_mask_value() {
    assert_eq!(RANK_1.0, 0x00000000000000FF);
}

#[test]
fn test_rank_8_mask_value() {
    assert_eq!(RANK_8.0, 0xFF00000000000000);
}

#[test]
fn test_rank_masks_array_length() {
    assert_eq!(RANK_MASKS.len(), 8); // assuming BOARD_SIZE = 8
}

#[test]
fn test_rank_masks_ordering() {
    let expected: [u64; 8] = [
        0x00000000000000FF,
        0x000000000000FF00,
        0x0000000000FF0000,
        0x00000000FF000000,
        0x000000FF00000000,
        0x0000FF0000000000,
        0x00FF000000000000,
        0xFF00000000000000,
    ];

    for (i, mask) in RANK_MASKS.iter().enumerate() {
        assert_eq!(mask.0, expected[i]);
    }
}
