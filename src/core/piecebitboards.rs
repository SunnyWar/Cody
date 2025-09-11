// src/core/piecebitbboards.rs

use crate::core::{bitboardmask::BitBoardMask, piece::Piece};

#[derive(Clone, Copy)]
pub struct PieceBitboards {
    inner: [BitBoardMask; 12],
}

impl PieceBitboards {
    pub const fn new() -> Self {
        Self {
            inner: [BitBoardMask(0); 12],
        }
    }

    #[inline]
    pub fn get(&self, piece: Piece) -> BitBoardMask {
        assert!(piece != Piece::None, "Tried to get() a None piece");
        self.inner[piece.index()]
    }

    #[inline]
    fn set(&mut self, piece: Piece, bb: BitBoardMask) {
        self.inner[piece.index()] = bb;
    }

    #[inline]
    pub fn all(&self) -> BitBoardMask {
        BitBoardMask(self.inner.iter().fold(0u64, |acc, bb| acc | bb.0))
    }

    pub fn get_mut(&mut self, piece: Piece) -> &mut BitBoardMask {
        &mut self.inner[piece.index()]
    }
}

impl Default for PieceBitboards {
    fn default() -> Self {
        Self::new()
    }
}

impl PieceBitboards {
    pub fn iter(&self) -> impl Iterator<Item = (Piece, BitBoardMask)> + '_ {
        self.inner
            .iter()
            .enumerate()
            .map(|(i, &bb)| (unsafe { std::mem::transmute::<u8, Piece>(i as u8) }, bb))
    }
}

impl PieceBitboards {
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Piece, &mut BitBoardMask)> {
        self.inner
            .iter_mut()
            .enumerate()
            .map(|(i, bb)| (unsafe { std::mem::transmute::<u8, Piece>(i as u8) }, bb))
    }
}
