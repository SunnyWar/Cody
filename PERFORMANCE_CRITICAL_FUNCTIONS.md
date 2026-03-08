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



| Priority | Module | Function | Call Freq | Impact | Primary Cost | Current Notes |
|---|---|---|---|---|---|---|
| 1 | Position | `copy_from()` | 100M/s | HIGH | ~100 cycles | Memcpy, Copy trait |
| 2 | Position | `all_pieces()` / `our_pieces()` | 10M/s | HIGH | ~1 cycle | Direct lookup |
| 3 | Movegen | `generate_pseudo_{knight,pawn,bishop,rook,queen,king}_moves_fast()` | 1M/s | HIGH | ~2-5k cycles | Piece-specific delegations |

## P2: Medium

| Priority | Module | Function | Call Freq | Impact | Primary Cost | Current Notes |
|---|---|---|---|---|---|---|
| 5 | Position | `piece_at()` | 100M/s | MEDIUM | ~1 cycle | Array indexing |
| 6 | Bitboard | `pawn_attacks_to()` | 100M/s | MEDIUM | ~1 cycle | Table lookup |
| 7 | Bitboards | `BitBoardMask::contains_square()` | 100M/s | MEDIUM | ~1 cycle | Bitwise AND |
| 8 | Bitboards | `BitBoardMask::count()` | 100M/s | MEDIUM | ~1 cycle | POPCNT |
| 9 | Intrinsics | `popcnt()` | 100M/s | MEDIUM | ~1 cycle | POPCNT instruction |
| 10 | Position | `to_board_state()` | 10M/s | MEDIUM | ~5k cycles | Piece reorganization |
| 11 | Eval | Mobility/King Safety/Rook Activity | 10M/s | MEDIUM | ~100-200 cycles | Bitboard iteration |
| 12 | SEE | `find_least_valuable_attacker()` | 10M/s | MEDIUM | ~100-1k cycles | Early exit on pawn |
| 13 | Bitboard | `king_attacks()` | 1M/s | MEDIUM | ~1 cycle | Table lookup |
| 14 | Bitboard | `knight_attacks()` | 1M/s | MEDIUM | ~1 cycle | Table lookup |
| 15 | SEE | `compute_see()` | 1M/s | MEDIUM | ~1-10k cycles | Recursive exchange |
| 16 | Zobrist | `compute_zobrist()` | 1M/s | MEDIUM | ~100-300 cycles | XOR piece keys |

---

## Next Target Recommendation

`copy_from()` is the next highest priority because it's called ~100M/s and has ~100-cycle cost across Position objects. Potential optimizations: bulk assignment patterns, Copy trait leverage, or inline memcpy.
