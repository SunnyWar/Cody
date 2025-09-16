// src/core/occupancy.rs

use std::ops::{Index, IndexMut};

use crate::BitBoardMask;

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
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: [BitBoardMask::empty(); 3],
        }
    }
    #[inline]
    pub fn or_in(&mut self, kind: OccupancyKind, mask: BitBoardMask) {
        self.inner[kind as usize] |= mask;
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner[0].is_empty() && self.inner[1].is_empty() && self.inner[2].is_empty()
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
