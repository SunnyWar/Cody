## Quick orientation for AI coding agents

This repository implements "Cody", a Rust chess engine focused on a deterministic, allocation-free fixed‑block search tree and a high‑performance pseudo‑legal move generator. The codebase is a Cargo workspace with two crates:

- `bitboard/` — pure bitboard logic, move generation, position representation and board utilities (no external deps). Key files: `bitboard/src/position.rs`, `bitboard/src/movegen.rs`, `bitboard/src/attack.rs`, `bitboard/src/piecebitboards.rs`, `bitboard/src/tables/*`.
- `engine/` — the search/engine layer, benches and UCI API. Key files: `engine/src/search/engine.rs`, `engine/src/core/arena.rs`, `engine/src/api/uciapi.rs`, `engine/benches/bench.rs`.

When making changes, prefer editing the smallest crate that contains the behavior (usually `bitboard` for board rules and `engine` for search/uci). The workspace root `Cargo.toml` binds them together.

## Big picture and important conventions

- Fixed‑block allocator and allocation‑free hot path: search nodes live in preallocated arenas and are recycled (see `engine/src/core/arena.rs` and `architecture.md`). Avoid heap allocations in hot loops. If adding data used in the hot path, prefer compact newtypes and integrate into the arena model.
- Move generation is pseudo‑legal (generates captures + quiet moves without legality filtering). Legality checks (e.g., in‑check) happen separately using `is_square_attacked` and board state snapshots (`Position::to_board_state()` in `bitboard/src/position.rs`). Prefer reusing `MoveGenContext` where present.
- Type safety and semantic newtypes: many types use small integer newtypes (e.g., `Ply`, `Depth`, `NodeId` in design docs). Preserve those semantics in public APIs.
- Small, explicit integer casts are used intentionally for performance and clarity (see `Move::new` patterns and `Position::from_fen` parsing). Maintain explicit casts instead of implicit conversions.

## Typical developer workflows (build, test, bench, debug)

- Build workspace (debug): `cargo build` from repo root.
- Build workspace (release, optimized for engine): `cargo build --release` (root profile sets LTO and codegen options in `Cargo.toml`).
- Run the engine CLI: `cargo run -p engine` (starts UCI API defined in `engine/src/main.rs`).
- Run benches: `cargo bench -p engine` (crate `engine` includes a bench harness; benches use `criterion`).
- Run unit tests per crate: `cargo test -p bitboard` or `cargo test -p engine`.

Notes:
- The repo keeps many build artifacts in `target/`. On Windows, use PowerShell for running commands. The workspace `Cargo.toml` uses resolver = "3" and custom release profile options — prefer `--release` when measuring performance.

## Project‑specific patterns and examples

- Position representation: `bitboard/src/position.rs` implements `Position` with `PieceBitboards` + `OccupancyMap`. Use `Position::apply_move_into(&self, &mv, &mut out)` to get a resulting position without mutating the source.
- FEN handling: `Position::from_fen` and `to_fen` are authoritative. Use these for tests and perft inputs (existing tests/benches use `engine/src/test_data.rs`).
- Move parsing: `Position::parse_uci_move` uses `generate_pseudo_moves(self)` to map algebraic coordinates to a `ChessMove`. Prefer that helper to constructing moves manually.
- Castling & EP: Castling rights live in `castling.rs` and are updated by `update_castling_rights` inside `position.rs`. En passant is represented as `Option<Square>` on `Position`.

## Integration points and dependencies

- `engine` depends on `bitboard` (path dependency in `engine/Cargo.toml`). Keep API changes in `bitboard` minimal and backward compatible when possible.
- External crates used by `engine`: `criterion` (benchmarking), `once_cell`, `rand`. The `bitboard` crate intentionally has no external deps.
- UCI API: implemented in `engine/src/api/uciapi.rs` and wired via `engine/src/main.rs`. Changes to command handling must maintain compatibility with existing UCI flows.

## How to modify safely (PR checklist for agents)

1. Run relevant unit tests for modified crate: `cargo test -p <crate>`.
2. If changes affect performance-sensitive code (movegen, apply_move, arena), run benchmarks: `cargo bench -p engine` and compare release builds with `--release`.
3. Preserve existing public APIs used across crates; update `engine` imports if `bitboard` signature changes.
4. When adding code in hot paths, keep allocations out of loops and avoid heap types like `Vec<T>` per-node; prefer `smallvec` or arena-backed storage if unavoidable.

## Quick examples to reference

- Apply a move without mutating the original:
  - `pos.apply_move_into(&mv, &mut out_pos)` (see `bitboard/src/position.rs`).
- Parse a UCI move string into an internal `ChessMove`:
  - `pos.parse_uci_move("e2e4")` which uses `generate_pseudo_moves(self)`.
- Test FEN round-trip:
  - `let p = Position::from_fen(fen); assert_eq!(p.to_fen(), fen);`

## What not to change without discussion

- Global search model (fixed‑block arena) and the allocation‑free hot path. These are core design constraints.
- Public move encoding shapes and bitboard layouts unless refactoring is coordinated across crates (`bitboard` → `engine`).

If something here is unclear or you need deeper examples (perft harnesses, bench scripts, or the arena API), tell me which area to expand and I'll iterate on this file.
