// tests/ranks_tests.rs

use cody::generated::{RANK_1, RANK_2, RANK_3, RANK_4, RANK_5, RANK_6, RANK_7, RANK_8, RANK_MASKS};

#[test]
fn test_rank_1_mask_value() {
    assert_eq!(RANK_1.0, 0x00000000000000FF);
}

#[test]
fn test_rank_2_mask_value() {
    assert_eq!(RANK_2.0, 0x000000000000FF00);
}

#[test]
fn test_rank_3_mask_value() {
    assert_eq!(RANK_3.0, 0x0000000000FF0000);
}

#[test]
fn test_rank_4_mask_value() {
    assert_eq!(RANK_4.0, 0x00000000FF000000);
}

#[test]
fn test_rank_5_mask_value() {
    assert_eq!(RANK_5.0, 0x000000FF00000000);
}

#[test]
fn test_rank_6_mask_value() {
    assert_eq!(RANK_6.0, 0x0000FF0000000000);
}

#[test]
fn test_rank_7_mask_value() {
    assert_eq!(RANK_7.0, 0x00FF000000000000);
}

#[test]
fn test_rank_8_mask_value() {
    assert_eq!(RANK_8.0, 0xFF00000000000000);
}

#[test]
fn test_rank_masks_array_length() {
    assert_eq!(RANK_MASKS.len(), 8);
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
