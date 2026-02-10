// src/search/engine.rs
// Thin compatibility shim: re-export main search implementation from
// `search.rs`
pub use crate::search::core::NODE_COUNT;
pub use crate::search::core::print_uci_info;
pub use crate::search::search::Engine;
