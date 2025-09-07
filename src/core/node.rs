use crate::core::position::Position;

#[derive(Clone)]
#[derive(Default)]
pub struct Node {
    pub position: Position,
    pub score: i32,
    pub children: Vec<usize>, // indices into Arena
}


