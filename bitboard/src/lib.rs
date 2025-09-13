// bitboard/src/lib.rs
#[derive(Clone, Copy)]
pub struct Bitboard(u64);

#[derive(Clone, Copy)]
pub struct Square(u8);

impl Bitboard {
    pub const EMPTY: Self = Bitboard(0);
    pub const fn from_u64(x: u64) -> Self { Bitboard(x) }
}

pub const fn rook_attacks(_sq: Square, _occ: Bitboard) -> Bitboard {
    // placeholder until you migrate your const-fn logic
    Bitboard::EMPTY
}
