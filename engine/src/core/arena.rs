// src/core/arena.rs

use crate::core::node::Node;

pub struct Arena {
    nodes: Vec<Node>,
    next_free: usize,
}

impl Arena {
    pub fn new(capacity: usize) -> Self {
        Self {
            nodes: vec![Node::default(); capacity],
            next_free: 0,
        }
    }

    pub fn alloc(&mut self) -> Option<usize> {
        if self.next_free >= self.nodes.len() {
            return None;
        }
        let idx = self.next_free;
        self.next_free += 1;
        Some(idx)
    }

    /// Returns an immutable reference to the node at `idx`.
    /// Bounds‐check is removed in release builds for maximum speed.
    #[inline(always)]
    pub fn get(&self, idx: usize) -> &Node {
        debug_assert!(idx < self.nodes.len());
        // SAFETY: caller guarantees `idx` is in-bounds.
        unsafe { self.nodes.get_unchecked(idx) }
    }

    /// Mutable counterpart of `get`.
    #[inline(always)]
    pub fn get_mut(&mut self, idx: usize) -> &mut Node {
        debug_assert!(idx < self.nodes.len());
        // SAFETY: caller guarantees `idx` is in-bounds and we have &mut self.
        unsafe { self.nodes.get_unchecked_mut(idx) }
    }

    pub fn reset(&mut self) {
        self.next_free = 0;
    }

    pub fn get_pair_mut(&mut self, idx1: usize, idx2: usize) -> (&Node, &mut Node) {
        debug_assert!(idx1 != idx2);
        let ptr = self.nodes.as_mut_ptr();
        // SAFETY: indices are distinct and in-bounds by caller contract.
        unsafe { (&*ptr.add(idx1), &mut *ptr.add(idx2)) }
    }
}
