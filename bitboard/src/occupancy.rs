// src/core/occupancy.rs

use crate::BitBoardMask;
use std::ops::Index;
use std::ops::IndexMut;

#[derive(Clone, Copy, PartialEq)]
pub enum OccupancyKind {
    White = 0,
    Black = 1,
    Both = 2,
}

#[derive(Clone, Copy, Debug)]
pub struct OccupancyMap {
    inner: [BitBoardMask; 3],
}

impl OccupancyMap {
    pub const fn new() -> Self {
        Self {
            inner: [BitBoardMask::empty(); 3],
        }
    }

    pub fn or_in(&mut self, kind: OccupancyKind, mask: BitBoardMask) {
        self.inner[kind as usize] |= mask;
    }

    pub const fn is_empty(&self) -> bool {
        self.inner[0].is_empty() && self.inner[1].is_empty() && self.inner[2].is_empty()
    }

    /// Direct accessor for all occupied squares (White | Black).
    /// Eliminates Index trait overhead by directly returning the cached union.
    pub fn get_both(&self) -> BitBoardMask {
        // Safety: OccupancyKind::Both == 2, always in-bounds.
        unsafe { *self.inner.get_unchecked(2) }
    }

    /// Direct accessor for one color's pieces.
    /// Uses Color discriminants (0=White, 1=Black) directly as array indices.
    /// Eliminates lookup table and Index trait overhead.
    pub fn get_by_color(&self, color: crate::piece::Color) -> BitBoardMask {
        // Safety: Color::White=0, Color::Black=1, both in-bounds for [0..3].
        unsafe { *self.inner.get_unchecked(color as usize) }
    }
}

impl Default for OccupancyMap {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<OccupancyKind> for OccupancyMap {
    type Output = BitBoardMask;

    fn index(&self, kind: OccupancyKind) -> &Self::Output {
        &self.inner[kind as usize]
    }
}

impl IndexMut<OccupancyKind> for OccupancyMap {
    fn index_mut(&mut self, kind: OccupancyKind) -> &mut Self::Output {
        &mut self.inner[kind as usize]
    }
}
