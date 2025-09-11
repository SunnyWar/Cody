// src/core/rank.rs
use crate::core::bitboardmask::BitBoardMask;

// Your const fn:
// rank = 0 → first rank (a1..h1)
// rank = 1 → second rank (a2..h2)
const fn rank_mask(rank: u8) -> BitBoardMask {
    let mut mask = 0u64;
    let mut file = 0;
    while file < 8 {
        mask |= 1u64 << (rank * 8 + file);
        file += 1;
    }
    BitBoardMask(mask)
}

const RANK_2_MASK: BitBoardMask = rank_mask(1);
const RANK_4_MASK: BitBoardMask = rank_mask(3);
const RANK_5_MASK: BitBoardMask = rank_mask(4);
const RANK_7_MASK: BitBoardMask = rank_mask(6);

const RANK_MASKS: [BitBoardMask; 8] = [
    rank_mask(0),
    rank_mask(1),
    rank_mask(2),
    rank_mask(3),
    rank_mask(4),
    rank_mask(5),
    rank_mask(6),
    rank_mask(7),
];

const EP_RANK_WHITE: BitBoardMask = rank_mask(5); // rank 6
const EP_RANK_BLACK: BitBoardMask = rank_mask(2); // rank 3
