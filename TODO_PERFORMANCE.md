# TODO List: Performance
Generated: 2026-02-15 04:25:18
**Stats**: 16 total | 4 not started | 5 in progress | 0 completed | 7 failed
---

## In Progress

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



### [ ] PERF-010: Reduce quiescence explosion via delta pruning and SEE-like filter
- **Priority**: critical
- **Category**: search
- **Complexity**: large
- **Files**: engine/src/search/quiescence.rs



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



## Not Started

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



## Failed

### [ ] PERF-001: Avoid repeated color piece ORs via cached occupancy bitboards
- **Priority**: high
- **Category**: memory
- **Complexity**: medium
- **Files**: bitboard/src/position.rs



*Completed: 2026-02-14T18:26:07.497242*

### [ ] PERF-004: Avoid repeated is_square_attacked calls in castling generation
- **Priority**: medium
- **Category**: move_gen
- **Complexity**: medium
- **Files**: bitboard/src/position.rs



*Completed: 2026-02-15T03:36:08.324669*

### [ ] PERF-005: Make occupancy_to_index use portable fallback and mark inline(always)
- **Priority**: medium
- **Category**: move_gen
- **Complexity**: medium
- **Files**: bitboard/src/bitboard.rs



*Completed: 2026-02-15T03:48:38.988397*

### [ ] PERF-008: Avoid Vec allocation for children in Node (Arena-based search)
- **Priority**: medium
- **Category**: memory
- **Complexity**: small
- **Files**: engine/src/core/node.rs, engine/src/core/arena.rs, engine/src/search/search.rs



*Completed: 2026-02-15T03:59:55.920058*

### [ ] PERF-009: Use TT best_move for root move ordering more aggressively
- **Priority**: high
- **Category**: search
- **Complexity**: medium
- **Files**: engine/src/search/search.rs



*Completed: 2026-02-15T02:46:22.490155*

### [ ] PERF-011: Use separate TT per thread to avoid contention and false sharing
- **Priority**: high
- **Category**: memory
- **Complexity**: large
- **Files**: engine/src/search/search.rs, engine/src/core/tt.rs



*Completed: 2026-02-15T02:56:21.834107*

### [ ] PERF-012: Avoid frequent file I/O and logging in hot search paths
- **Priority**: medium
- **Category**: search
- **Complexity**: medium
- **Files**: engine/src/search/core.rs, engine/src/search/quiescence.rs, engine/src/api/uciapi.rs



*Completed: 2026-02-15T04:10:29.866795*
