// src/core/node.rs

use bitboard::position::Position;

#[derive(Clone, Default)]
pub struct Node {
    pub position: Position,
    pub score: i32,
    pub children: Vec<usize>, // indices into Arena
}
