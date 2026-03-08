// bitboard/src/movegen/mod.rs
// Module entry for move generation. Public API is re-exported from `api`.

pub mod api;
pub mod captures;
pub mod king;
pub mod knight;
pub mod legality;
pub mod pawn;
pub mod sliders;

pub use api::MoveGenerator;
pub use api::SimpleMoveGen;
pub use api::generate_legal_moves;
pub use api::generate_legal_moves_fast;
pub use api::generate_pseudo_captures_fast;
pub use api::generate_pseudo_moves_fast;
pub use api::validate_legal_move_generation;
pub use king::generate_pseudo_king_moves;
pub use king::generate_pseudo_king_moves_fast;
pub use knight::generate_pseudo_knight_moves;
pub use knight::generate_pseudo_knight_moves_fast;
pub use legality::is_in_check;
pub use legality::is_legal;
pub use legality::is_legal_fast;
pub use pawn::generate_pseudo_pawn_moves;
pub use pawn::generate_pseudo_pawn_moves_fast;
pub use sliders::generate_pseudo_bishop_moves;
pub use sliders::generate_pseudo_bishop_moves_fast;
pub use sliders::generate_pseudo_queen_moves;
pub use sliders::generate_pseudo_queen_moves_fast;
pub use sliders::generate_pseudo_rook_moves;
pub use sliders::generate_pseudo_rook_moves_fast;
