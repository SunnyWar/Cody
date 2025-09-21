// bitboard/src/movegen/mod.rs
// Module entry for move generation. Public API is re-exported from `api`.

pub mod api;
pub mod captures;
pub mod king;
pub mod knight;
pub mod legality;
pub mod pawn;
pub mod sliders;

pub use api::{
    MoveGenerator, SimpleMoveGen, generate_legal_moves, generate_pseudo_captures,
    generate_pseudo_moves,
};

// Re-export pawn generator to preserve the original API surface
pub use king::generate_pseudo_king_moves;
pub use knight::generate_pseudo_knight_moves;
pub use legality::is_legal;
pub use pawn::generate_pseudo_pawn_moves;
pub use sliders::{
    generate_pseudo_bishop_moves, generate_pseudo_queen_moves, generate_pseudo_rook_moves,
};
