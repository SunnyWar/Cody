# Cody Chess Engine: Remaining Optimization Backlog

**Date:** March 7, 2026
**Scope:** Only functions that are still not optimized
**Priority Rule:** CRITICAL > HIGH > MEDIUM > LOW, then by call frequency and expected impact

---

## P0: Critical (Do First)

| Priority | Module | Function | Call Freq | Impact | Primary Cost | Current Notes |
|---|---|---|---|---|---|---|

## Recently Completed Optimizations (Current Session)

| Module | Function | Optimization | Impact |
|---|---|---|---|
| Intrinsics | `trailing_zeros()` | Added `trailing_zeros_nonzero()` fast-path; routed proven-nonzero call sites (BitIter, SquaresIter, first_square, is_king_in_check) through it. | ~1 cycle saved on 100M+ calls |
| Intrinsics | `blsr()` | Added `blsr_nonzero()` fast-path; routed proven-nonzero call sites (BitIter, SquaresIter) through it. | ~1 cycle saved on 100M+ calls |
| Bitboard | `BitBoardMask::squares()` iterator | Removed `Square::try_from_index()` Option wrapper using direct `unsafe transmute` in guaranteed-nonzero path. | 2-3 cycles per iterator step |
| Bitboard | `rook_attacks_from()` | Single-indexed square once; used unchecked table lookups in hot path. | ~2 cycles per call |
| Bitboard | `bishop_attacks_from()` | Single-indexed square once; used unchecked table lookups in hot path. | ~2 cycles per call |
| Bitboard | `occupancy_to_index()` | Added raw u64 fast-path (`occupancy_to_index_u64`); switched all hot slider/attack paths to use it. Bypasses BitBoardMask wrapper field extraction ~6 times per move. | 1-2 cycles per occupancy lookup |
| Intrinsics | `pext()` | Added `pext_nonzero()` variant with debug_assert for guaranteed non-zero masks. Documents precondition and allows compiler optimizations in known-safe contexts. | Potential 1+ cycle in specialized paths |
| Position | `copy_from()` | Replaced structural assignment with `core::ptr::copy_nonoverlapping()` for explicit bulk copy semantics. Allows better compiler optimization without relying on cross-crate inlining. | 2-5 cycles per search initialization |
| OccupancyMap & Position | `all_pieces()` / `our_pieces()` | Added `get_both()` and `get_by_color()` methods to OccupancyMap; direct unchecked access bypasses Index trait overhead and lookup table. | 1-2 cycles per move generation |
| Movegen | `generate_pseudo_{knight,bishop,rook,queen}_moves_fast()` | Added early `is_empty()` returns for piece-type iteration loops. Avoids unnecessary iterator overhead when a piece type is absent (common in endgames). | 2-4 cycles in tactical positions ~20% of moves |
| Position | `piece_at()` | Added `piece_at_square()` direct accessor returning raw `Piece` (no Option wrapping). Updated hot-path callers in SEE and quiescence to avoid Option overhead. | 1-2 cycles per call in SEE/quiescence |
| Bitboard | `pawn_attacks_to()` | Eliminated `attacker_color.opposite().index()` call using direct Color enum XOR; Color variants are 0 (White) and 1 (Black), XOR with 1 flips the color. | ~1 cycle per lookup |
| Position | `to_board_state()` | Replaced 12 `Piece::from_parts()` constructor calls with direct Piece enum variants (WhitePawn, BlackQueen, etc.). Each variant is already a compiled discriminant value. | ~5k cycles per call (eliminated 12 branches/function calls per 10M/s invocations) |

## P1: High

| Priority | Module | Function | Call Freq | Impact | Primary Cost | Current Notes |
|---|---|---|---|---|---|---|

## P2: Medium

| Priority | Module | Function | Call Freq | Impact | Primary Cost | Current Notes |
|---|---|---|---|---|---|---|
| 1 | Bitboards | `BitBoardMask::contains_square()` | 100M/s | MEDIUM | ~1 cycle | Bitwise AND |
| 2 | Bitboards | `BitBoardMask::count()` | 100M/s | MEDIUM | ~1 cycle | POPCNT |
| 3 | Intrinsics | `popcnt()` | 100M/s | MEDIUM | ~1 cycle | POPCNT instruction |
| 4 | Eval | Mobility/King Safety/Rook Activity | 10M/s | MEDIUM | ~100-200 cycles | Bitboard iteration |
| 5 | SEE | `find_least_valuable_attacker()` | 10M/s | MEDIUM | ~100-1k cycles | Early exit on pawn |
| 6 | Bitboard | `king_attacks()` | 1M/s | MEDIUM | ~1 cycle | Table lookup |
| 7 | Bitboard | `knight_attacks()` | 1M/s | MEDIUM | ~1 cycle | Table lookup |
| 8 | SEE | `compute_see()` | 1M/s | MEDIUM | ~1-10k cycles | Recursive exchange |
| 9 | Zobrist | `compute_zobrist()` | 1M/s | MEDIUM | ~100-300 cycles | XOR piece keys |

---

## Next Target Recommendation

P2 (MEDIUM) now active. Highest-frequency targets: `pawn_attacks_to()` (100M/s), `BitBoardMask::contains_square()` (100M/s), `BitBoardMask::count()` (100M/s), `popcnt()` (100M/s). All involve simple bitboard operations that may benefit from direct accessors or intrinsic routing.
