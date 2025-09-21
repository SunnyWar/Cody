// bitboard/src/movegen/mod.rs
// Module entry for move generation. Public API is re-exported from `api`.

pub mod api;
pub mod pawn;

pub use api::{
    MoveGenerator, SimpleMoveGen, generate_legal_moves, generate_pseudo_captures,
    generate_pseudo_moves, is_legal,
};

// Re-export pawn generator to preserve the original API surface
pub use pawn::generate_pseudo_pawn_moves;
