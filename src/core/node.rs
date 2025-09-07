use crate::core::position::Position;

#[derive(Clone)]
pub struct Node {
    pub position: Position,
    pub score: i32,
    pub children: Vec<usize>, // indices into Arena
}

impl Default for Node {
    fn default() -> Self {
        Self {
            position: Position::default(),
            score: 0,
            children: Vec::new(),
        }
    }
}
