// bitboard/src/piecebitbboards.rs

use crate::BitBoardMask;
use crate::piece::Piece;

#[derive(Clone, Copy, Debug)]
#[repr(align(64))]
pub struct PieceBitboards {
    inner: [BitBoardMask; 12],
}

impl PieceBitboards {
    pub const fn new() -> Self {
        Self {
            inner: [BitBoardMask(0); 12],
        }
    }

    /// Hot-path accessor: force inlining, strip runtime checks and
    /// bounds-checks.

    pub fn get(&self, piece: Piece) -> BitBoardMask {
        // Keep the correctness guard in debug builds without polluting release code.
        debug_assert!(piece != Piece::None, "Tried to get() a None piece");

        // Safety: `piece.index()` is guaranteed to be within the bounds of `inner`
        // for all valid `Piece` discriminants.
        unsafe { *self.inner.get_unchecked(piece.index()) }
    }

    /// Fast union of every piece bitboard.
    ///
    /// A manual `for` loop avoids the iterator/closure machinery used by
    /// `fold`, giving the compiler freedom to unroll/vectorise this fixed
    /// 12-element scan.  This shaves a few instructions off a call that is
    /// executed millions of times per search.
    pub fn all(&self) -> BitBoardMask {
        let mut acc = 0u64;
        for bb in &self.inner {
            acc |= bb.0;
        }
        BitBoardMask(acc)
    }

    pub const fn get_mut(&mut self, piece: Piece) -> &mut BitBoardMask {
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
