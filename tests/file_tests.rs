// tests/file_tests.rs

use cody::generated::{
    FILE_A, FILE_B, FILE_C, FILE_D, FILE_E, FILE_F, FILE_G, FILE_H, FILE_MASKS, NOT_FILE_A,
    NOT_FILE_AB, NOT_FILE_GH, NOT_FILE_H,
};

#[test]
fn test_file_a_mask_value() {
    assert_eq!(FILE_A.0, 0x0101010101010101);
}

#[test]
fn test_file_b_mask_value() {
    assert_eq!(FILE_B.0, 0x0202020202020202);
}

#[test]
fn test_file_c_mask_value() {
    assert_eq!(FILE_C.0, 0x0404040404040404);
}

#[test]
fn test_file_d_mask_value() {
    assert_eq!(FILE_D.0, 0x0808080808080808);
}

#[test]
fn test_file_e_mask_value() {
    assert_eq!(FILE_E.0, 0x1010101010101010);
}

#[test]
fn test_file_f_mask_value() {
    assert_eq!(FILE_F.0, 0x2020202020202020);
}

#[test]
fn test_file_g_mask_value() {
    assert_eq!(FILE_G.0, 0x4040404040404040);
}

#[test]
fn test_file_h_mask_value() {
    assert_eq!(FILE_H.0, 0x8080808080808080);
}

#[test]
fn test_file_masks_array_length() {
    assert_eq!(FILE_MASKS.len(), 8);
}

#[test]
fn test_file_masks_ordering() {
    let expected: [u64; 8] = [
        0x0101010101010101,
        0x0202020202020202,
        0x0404040404040404,
        0x0808080808080808,
        0x1010101010101010,
        0x2020202020202020,
        0x4040404040404040,
        0x8080808080808080,
    ];

    for (i, mask) in FILE_MASKS.iter().enumerate() {
        assert_eq!(mask.0, expected[i]);
    }
}

#[test]
fn test_not_file_a_mask() {
    let expected: u64 = 0xFEFEFEFEFEFEFEFE;
    assert_eq!(NOT_FILE_A.0, expected);
}

#[test]
fn test_not_file_ab_mask() {
    let expected: u64 = 0xFCFCFCFCFCFCFCFC;
    assert_eq!(NOT_FILE_AB.0, expected);
}

#[test]
fn test_not_file_h_mask() {
    let expected: u64 = 0x7F7F7F7F7F7F7F7F;
    assert_eq!(NOT_FILE_H.0, expected);
}

#[test]
fn test_not_file_gh_mask() {
    let expected: u64 = 0x3F3F3F3F3F3F3F3F;
    assert_eq!(NOT_FILE_GH.0, expected);
}
