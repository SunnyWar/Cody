# TODO List: Performance
Generated: 2026-02-14 17:59:42
**Stats**: 16 total | 15 not started | 1 in progress | 0 completed | 0 failed
---

## In Progress

### [ ] PERF-010: Reduce quiescence explosion via delta pruning and SEE-like filter
- **Priority**: critical
- **Category**: search
- **Complexity**: large
- **Files**: engine/src/search/quiescence.rs



## Not Started

### [ ] PERF-001: Avoid repeated color piece ORs via cached occupancy bitboards
- **Priority**: high
- **Category**: memory
- **Complexity**: medium
- **Files**: bitboard/src/position.rs



### [ ] PERF-002: Inline and simplify BitBoardMask helpers to reduce overhead
- **Priority**: medium
- **Category**: rust_specific
- **Complexity**: small
- **Files**: bitboard/src/bitboardmask.rs



### [ ] PERF-003: Remove O(64) Square::all_array scans in generate_pseudo_captures
- **Priority**: high
- **Category**: move_gen
- **Complexity**: medium
- **Files**: bitboard/src/movegen/captures.rs



### [ ] PERF-004: Avoid repeated is_square_attacked calls in castling generation
- **Priority**: medium
- **Category**: move_gen
- **Complexity**: medium
- **Files**: bitboard/src/position.rs



### [ ] PERF-005: Make occupancy_to_index use portable fallback and mark inline(always)
- **Priority**: medium
- **Category**: move_gen
- **Complexity**: medium
- **Files**: bitboard/src/bitboard.rs



### [ ] PERF-006: Simplify subray diagonal functions using precomputed masks
- **Priority**: low
- **Category**: algorithmic
- **Complexity**: large
- **Files**: bitboard/src/bitboardmask.rs, bitboard/src/bitboard.rs



### [ ] PERF-007: Speed up Position::from_fen with fixed-size arrays and no Vec
- **Priority**: low
- **Category**: rust_specific
- **Complexity**: medium
- **Files**: bitboard/src/position.rs



### [ ] PERF-008: Avoid Vec allocation for children in Node (Arena-based search)
- **Priority**: medium
- **Category**: memory
- **Complexity**: small
- **Files**: engine/src/core/node.rs, engine/src/core/arena.rs, engine/src/search/search.rs



### [ ] PERF-009: Use TT best_move for root move ordering more aggressively
- **Priority**: high
- **Category**: search
- **Complexity**: medium
- **Files**: engine/src/search/search.rs



### [ ] PERF-011: Use separate TT per thread to avoid contention and false sharing
- **Priority**: high
- **Category**: memory
- **Complexity**: large
- **Files**: engine/src/search/search.rs, engine/src/core/tt.rs



### [ ] PERF-012: Avoid frequent file I/O and logging in hot search paths
- **Priority**: medium
- **Category**: search
- **Complexity**: medium
- **Files**: engine/src/search/core.rs, engine/src/search/quiescence.rs, engine/src/api/uciapi.rs



### [ ] PERF-013: Use LRU-like replacement and aging in TT store policy
- **Priority**: low
- **Category**: search
- **Complexity**: large
- **Files**: engine/src/core/tt.rs



### [ ] PERF-014: Leverage const fn more widely for precomputed tables and masks
- **Priority**: low
- **Category**: rust_specific
- **Complexity**: medium
- **Files**: bitboard/src/bitboard.rs, bitboard/src/bitboardmask.rs, bitboard/src/square.rs, bitboard/src/tables/*.rs



### [ ] PERF-015: Enable LTO and tune codegen-units for search crate
- **Priority**: high
- **Category**: compilation
- **Complexity**: small
- **Files**: Cargo.toml



### [ ] PERF-016: Apply PGO to focus optimization on search hotspots
- **Priority**: medium
- **Category**: compilation
- **Complexity**: large
- **Files**: Cargo.toml, engine/benches/bench.rs


