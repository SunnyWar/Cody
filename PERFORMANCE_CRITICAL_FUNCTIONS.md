# Cody Chess Engine: Remaining Optimization Backlog

**Date:** March 7, 2026
**Scope:** Only functions that are still not optimized
**Priority Rule:** CRITICAL > HIGH > MEDIUM > LOW, then by call frequency and expected impact

---

## P0: Critical (Do First)

| Priority | Module | Function | Call Freq | Impact | Primary Cost | Current Notes |
|---|---|---|---|---|---|---|
| 1 | Intrinsics | `trailing_zeros()` | 1B/s | CRITICAL | ~1 cycle | TZCNT/BMI1 |
| 2 | Intrinsics | `blsr()` | 1B/s | CRITICAL | ~1 cycle | BLSR/BMI1 |
| 3 | Bitboards | `BitBoardMask::squares()` iterator | 1B/s | CRITICAL | ~per-bit | Trailing zeros + BLSR |

## P1: High

| Priority | Module | Function | Call Freq | Impact | Primary Cost | Current Notes |
|---|---|---|---|---|---|---|
| 4 | Bitboard | `rook_attacks_from()` | 100M/s | HIGH | ~3 cycles | Magic bitboard + PEXT |
| 5 | Bitboard | `bishop_attacks_from()` | 100M/s | HIGH | ~3 cycles | Magic bitboard + PEXT |
| 6 | Bitboard | `occupancy_to_index()` | 100M/s | HIGH | ~1 cycle | PEXT instruction |
| 7 | Intrinsics | `pext()` | 100M/s | HIGH | ~1 cycle | PEXT/BMI2 |
| 8 | Position | `copy_from()` | 100M/s | HIGH | ~100 cycles | Memcpy, Copy trait |
| 9 | Position | `all_pieces()` / `our_pieces()` | 10M/s | HIGH | ~1 cycle | Direct lookup |
| 10 | Movegen | `generate_pseudo_{knight,pawn,bishop,rook,queen,king}_moves_fast()` | 1M/s | HIGH | ~2-5k cycles | Piece-specific delegations |

## P2: Medium

| Priority | Module | Function | Call Freq | Impact | Primary Cost | Current Notes |
|---|---|---|---|---|---|---|
| 11 | Position | `piece_at()` | 100M/s | MEDIUM | ~1 cycle | Array indexing |
| 12 | Bitboard | `pawn_attacks_to()` | 100M/s | MEDIUM | ~1 cycle | Table lookup |
| 13 | Bitboards | `BitBoardMask::contains_square()` | 100M/s | MEDIUM | ~1 cycle | Bitwise AND |
| 14 | Bitboards | `BitBoardMask::count()` | 100M/s | MEDIUM | ~1 cycle | POPCNT |
| 15 | Intrinsics | `popcnt()` | 100M/s | MEDIUM | ~1 cycle | POPCNT instruction |
| 16 | Position | `to_board_state()` | 10M/s | MEDIUM | ~5k cycles | Piece reorganization |
| 17 | Eval | Mobility/King Safety/Rook Activity | 10M/s | MEDIUM | ~100-200 cycles | Bitboard iteration |
| 18 | SEE | `find_least_valuable_attacker()` | 10M/s | MEDIUM | ~100-1k cycles | Early exit on pawn |
| 19 | Bitboard | `king_attacks()` | 1M/s | MEDIUM | ~1 cycle | Table lookup |
| 20 | Bitboard | `knight_attacks()` | 1M/s | MEDIUM | ~1 cycle | Table lookup |
| 21 | SEE | `compute_see()` | 1M/s | MEDIUM | ~1-10k cycles | Recursive exchange |
| 22 | Zobrist | `compute_zobrist()` | 1M/s | MEDIUM | ~100-300 cycles | XOR piece keys |

---

## Next Target Recommendation

`trailing_zeros()` is the highest priority remaining target because it is CRITICAL and called at ~1B/s.
