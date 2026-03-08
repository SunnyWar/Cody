# Cody Chess Engine: Remaining Optimization Backlog

**Date:** March 7, 2026
**Scope:** Only functions that are still not optimized
**Priority Rule:** CRITICAL > HIGH > MEDIUM > LOW, then by call frequency and expected impact

---

## P0: Critical (Do First)

| Priority | Module | Function | Call Freq | Impact | Primary Cost | Current Notes |
|---|---|---|---|---|---|---|
| 1 | Intrinsics | `blsr()` | 1B/s | CRITICAL | ~1 cycle | BLSR/BMI1 |
| 2 | Bitboards | `BitBoardMask::squares()` iterator | 1B/s | CRITICAL | ~per-bit | Trailing zeros + BLSR |

## P1: High

| Priority | Module | Function | Call Freq | Impact | Primary Cost | Current Notes |
|---|---|---|---|---|---|---|
| 3 | Bitboard | `rook_attacks_from()` | 100M/s | HIGH | ~3 cycles | Magic bitboard + PEXT |
| 4 | Bitboard | `bishop_attacks_from()` | 100M/s | HIGH | ~3 cycles | Magic bitboard + PEXT |
| 5 | Bitboard | `occupancy_to_index()` | 100M/s | HIGH | ~1 cycle | PEXT instruction |
| 6 | Intrinsics | `pext()` | 100M/s | HIGH | ~1 cycle | PEXT/BMI2 |
| 7 | Position | `copy_from()` | 100M/s | HIGH | ~100 cycles | Memcpy, Copy trait |
| 8 | Position | `all_pieces()` / `our_pieces()` | 10M/s | HIGH | ~1 cycle | Direct lookup |
| 9 | Movegen | `generate_pseudo_{knight,pawn,bishop,rook,queen,king}_moves_fast()` | 1M/s | HIGH | ~2-5k cycles | Piece-specific delegations |

## P2: Medium

| Priority | Module | Function | Call Freq | Impact | Primary Cost | Current Notes |
|---|---|---|---|---|---|---|
| 10 | Position | `piece_at()` | 100M/s | MEDIUM | ~1 cycle | Array indexing |
| 11 | Bitboard | `pawn_attacks_to()` | 100M/s | MEDIUM | ~1 cycle | Table lookup |
| 12 | Bitboards | `BitBoardMask::contains_square()` | 100M/s | MEDIUM | ~1 cycle | Bitwise AND |
| 13 | Bitboards | `BitBoardMask::count()` | 100M/s | MEDIUM | ~1 cycle | POPCNT |
| 14 | Intrinsics | `popcnt()` | 100M/s | MEDIUM | ~1 cycle | POPCNT instruction |
| 15 | Position | `to_board_state()` | 10M/s | MEDIUM | ~5k cycles | Piece reorganization |
| 16 | Eval | Mobility/King Safety/Rook Activity | 10M/s | MEDIUM | ~100-200 cycles | Bitboard iteration |
| 17 | SEE | `find_least_valuable_attacker()` | 10M/s | MEDIUM | ~100-1k cycles | Early exit on pawn |
| 18 | Bitboard | `king_attacks()` | 1M/s | MEDIUM | ~1 cycle | Table lookup |
| 19 | Bitboard | `knight_attacks()` | 1M/s | MEDIUM | ~1 cycle | Table lookup |
| 20 | SEE | `compute_see()` | 1M/s | MEDIUM | ~1-10k cycles | Recursive exchange |
| 21 | Zobrist | `compute_zobrist()` | 1M/s | MEDIUM | ~100-300 cycles | XOR piece keys |

---

## Next Target Recommendation

`blsr()` is the highest priority remaining target because it is CRITICAL and called at ~1B/s.
