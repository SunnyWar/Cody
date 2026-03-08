#[allow(clippy::module_inception)]
pub mod search;

pub mod engine;
pub mod evaluator;
pub mod piecesquaretable;
pub mod quiescence;
pub mod see;
pub mod tablebase;

// The main search implementation lives in `search.rs` but we keep the
// `engine` shim for compatibility. Expose it as a sibling module name
// `search_impl`.
mod core;

pub use core::*;
