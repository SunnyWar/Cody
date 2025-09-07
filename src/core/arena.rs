/// Fixed-block memory arena for search nodes.
/// Phase 1: focus on allocation/recycling speed, not chess logic.
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

    pub fn get(&self, idx: usize) -> &Node {
        &self.nodes[idx]
    }

    pub fn get_mut(&mut self, idx: usize) -> &mut Node {
        &mut self.nodes[idx]
    }

    pub fn reset(&mut self) {
        self.next_free = 0;
    }
}
