// src/core/bitboard.rs
#[inline]
pub const fn bit(sq: u8) -> u64 {
    1u64 << (sq as u32)
}

pub struct BitIter(u64);

impl Iterator for BitIter {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            None
        } else {
            let sq = self.0.trailing_zeros() as u8;
            self.0 &= self.0 - 1; // clear LS1B
            Some(sq)
        }
    }
}

pub fn bit_iter(bb: u64) -> BitIter {
    BitIter(bb)
}

#[inline]
pub fn knight_attacks(sq: u8) -> u64 {
    let b = bit(sq);
    let l1 = (b >> 1) & 0x7f7f7f7f7f7f7f7f;
    let l2 = (b >> 2) & 0x3f3f3f3f3f3f3f3f;
    let r1 = (b << 1) & 0xfefefefefefefefe;
    let r2 = (b << 2) & 0xfcfcfcfcfcfcfcfc;
    let h1 = l1 | r1;
    let h2 = l2 | r2;
    (h1 << 16) | (h1 >> 16) | (h2 << 8) | (h2 >> 8)
}

