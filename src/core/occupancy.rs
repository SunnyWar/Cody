// src/core/occupancy.rs

use crate::core::bitboardmask::BitBoardMask;
use std::ops::{Index, IndexMut};

#[derive(Clone, Copy, PartialEq)]
pub enum OccupancyKind {
    White = 0,
    Black = 1,
    Both = 2,
}

#[derive(Clone, Copy)]
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
    fn get(&self, kind: OccupancyKind) -> BitBoardMask {
        self.inner[kind as usize]
    }

    #[inline]
    fn set(&mut self, kind: OccupancyKind, value: BitBoardMask) {
        self.inner[kind as usize] = value;
    }

    #[inline]
    pub fn or_in(&mut self, kind: OccupancyKind, mask: BitBoardMask) {
        self.inner[kind as usize] |= mask;
    }

    #[inline]
    fn clear(&mut self, kind: OccupancyKind) {
        self.inner[kind as usize] = BitBoardMask::empty();
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
