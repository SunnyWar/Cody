pub mod search;
// src/search/mod.rs

pub mod engine;
pub mod evaluator;
pub mod piecesquaretable;
pub mod quiescence;

// The main search implementation lives in `search.rs` but we keep the
// `engine` shim for compatibility. Expose it as a sibling module name
// `search_impl`.
pub mod core;
