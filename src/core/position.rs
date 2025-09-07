/// Minimal placeholder for board state.
/// Later: implement full chess rules.
#[derive(Clone)]
pub struct Position {
    pub board: [u8; 64],
    pub side_to_move: bool,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            board: [0; 64],
            side_to_move: true,
        }
    }
}

impl Position {
    pub fn generate_legal_moves(&self) -> Vec<Move> {
        // Phase 1: stub
        Vec::new()
    }

    pub fn apply_move(&self, _mv: &Move) -> Position {
        // Phase 1: stub
        self.clone()
    }
}

#[derive(Clone)]
pub struct Move {
    pub from: u8,
    pub to: u8,
}
