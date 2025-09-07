// src/core/bitboard.rs
#[inline]
pub const fn bit(sq: u8) -> u64 {
    1u64 << (sq as u32)
}
