# Cody Chess Engine: Remaining Optimization Backlog

**Date:** March 7, 2026
**Scope:** Only functions that are still not optimized
**Priority Rule:** CRITICAL > HIGH > MEDIUM > LOW, then by call frequency and expected impact

---

## P0: Critical (Do First)

| Priority | Module | Function | Call Freq | Impact | Primary Cost | Current Notes |
|---|---|---|---|---|---|---|
| 1 | Bitboards | `BitBoardMask::squares()` iterator | 1B/s | CRITICAL | ~per-bit | Trailing zeros + BLSR |

## P1: High

| Priority | Module | Function | Call Freq | Impact | Primary Cost | Current Notes |
|---|---|---|---|---|---|---|
| 2 | Bitboard | `rook_attacks_from()` | 100M/s | HIGH | ~3 cycles | Magic bitboard + PEXT |
| 3 | Bitboard | `bishop_attacks_from()` | 100M/s | HIGH | ~3 cycles | Magic bitboard + PEXT |
| 4 | Bitboard | `occupancy_to_index()` | 100M/s | HIGH | ~1 cycle | PEXT instruction |
| 5 | Intrinsics | `pext()` | 100M/s | HIGH | ~1 cycle | PEXT/BMI2 |
| 6 | Position | `copy_from()` | 100M/s | HIGH | ~100 cycles | Memcpy, Copy trait |
| 7 | Position | `all_pieces()` / `our_pieces()` | 10M/s | HIGH | ~1 cycle | Direct lookup |
| 8 | Movegen | `generate_pseudo_{knight,pawn,bishop,rook,queen,king}_moves_fast()` | 1M/s | HIGH | ~2-5k cycles | Piece-specific delegations |

## P2: Medium

| Priority | Module | Function | Call Freq | Impact | Primary Cost | Current Notes |
|---|---|---|---|---|---|---|
| 9 | Position | `piece_at()` | 100M/s | MEDIUM | ~1 cycle | Array indexing |
| 10 | Bitboard | `pawn_attacks_to()` | 100M/s | MEDIUM | ~1 cycle | Table lookup |
| 11 | Bitboards | `BitBoardMask::contains_square()` | 100M/s | MEDIUM | ~1 cycle | Bitwise AND |
| 12 | Bitboards | `BitBoardMask::count()` | 100M/s | MEDIUM | ~1 cycle | POPCNT |
| 13 | Intrinsics | `popcnt()` | 100M/s | MEDIUM | ~1 cycle | POPCNT instruction |
| 14 | Position | `to_board_state()` | 10M/s | MEDIUM | ~5k cycles | Piece reorganization |
| 15 | Eval | Mobility/King Safety/Rook Activity | 10M/s | MEDIUM | ~100-200 cycles | Bitboard iteration |
| 16 | SEE | `find_least_valuable_attacker()` | 10M/s | MEDIUM | ~100-1k cycles | Early exit on pawn |
| 17 | Bitboard | `king_attacks()` | 1M/s | MEDIUM | ~1 cycle | Table lookup |
| 18 | Bitboard | `knight_attacks()` | 1M/s | MEDIUM | ~1 cycle | Table lookup |
| 19 | SEE | `compute_see()` | 1M/s | MEDIUM | ~1-10k cycles | Recursive exchange |
| 20 | Zobrist | `compute_zobrist()` | 1M/s | MEDIUM | ~100-300 cycles | XOR piece keys |

---

## Next Target Recommendation

`BitBoardMask::squares()` iterator is the highest priority remaining target because it is CRITICAL and called at ~1B/s.
