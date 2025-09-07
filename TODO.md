# Cody Chess Engine — MVP TODO Roadmap

## 1. Board Representation & State Management
**Goal:** A complete `Position` struct that can represent any legal chess position.  
**Tasks:**
- Choose representation: **bitboards** (fast) or **mailbox 0x88** (simpler to debug).
- Store:
  - Piece placement
  - Side to move
  - Castling rights (4 bits)
  - En passant target square
  - Halfmove clock (50‑move rule)
  - Fullmove number
- Implement:
  - `Position::from_fen(&str)` → parse FEN into internal state.
  - `Position::to_fen()` → serialize back to FEN.
- Add `debug_print()` for human‑readable board dumps.

---

## 2. Move Representation
**Goal:** A compact `Move` type encoding all necessary info.  
**Tasks:**
- Store: from‑square, to‑square, promotion piece, flags (capture, castle, en passant).
- Implement helper constructors: `Move::new()`, `Move::promotion()`, etc.
- Implement `Display` for algebraic or UCI notation.

---

## 3. Move Generation (Pseudo‑Legal)
**Goal:** Generate all moves without checking king safety.  
**Tasks:**
- Implement per‑piece move generation:
  - Pawns (push, capture, promotion, en passant)
  - Knights
  - Bishops
  - Rooks
  - Queens
  - King (including castling)
- Return moves in `Vec<Move>` or preallocated buffer.
- Count moves per piece type for profiling.

---

## 4. Legal Move Filtering
**Goal:** Remove moves that leave own king in check.  
**Tasks:**
- Implement `Position::in_check(side)`.
- For each pseudo‑legal move:
  - Apply move to a copy of the position.
  - Discard if king is in check.

---

## 5. Move Application & Undo
**Goal:** Correctly update position state for search.  
**Tasks:**
- Implement `apply_move(&mut self, mv: Move)` or return new `Position`.
- Maintain move history stack for undo.
- Update:
  - Piece placement
  - Castling rights
  - En passant square
  - Halfmove/fullmove counters
- Verify state transitions with FEN before/after.

---

## 6. Perft (Movegen Validation)
**Goal:** Ensure move generation is correct before search.  
**Tasks:**
- Implement `perft(depth)` to count all legal positions at given depth.
- Compare against known perft values.
- Log node counts per depth; stop if mismatch.

---

## 7. Search Core (Negamax + Alpha‑Beta)
**Goal:** Basic minimax search with pruning.  
**Tasks:**
- Implement `negamax(position, depth, alpha, beta) -> score`.
- Terminal conditions:
  - Depth = 0 → return evaluation.
  - No legal moves → checkmate/stalemate scoring.
- Add alpha‑beta pruning.
- Count nodes, cutoffs, max depth reached.

---

## 8. Evaluation Function (Material‑Only MVP)
**Goal:** Assign a score to a position.  
**Tasks:**
- Piece values: Pawn=100, Knight=320, Bishop=330, Rook=500, Queen=900.
- Score = (material for side to move) − (material for opponent).
- Log score breakdown by piece type.

---

## 9. Iterative Deepening (Optional for MVP)
**Goal:** Improve move ordering and allow time control.  
**Tasks:**
- Search depth 1, then 2, etc., keeping best move so far.
- Stop when time runs out.

---

## 10. UCI Protocol Implementation
**Goal:** Make Cody playable in GUIs like Arena or CuteChess.  
**Tasks:**
- Implement commands:
  - `uci` → print engine info
  - `isready` → `readyok`
  - `position [fen|startpos] moves ...`
  - `go depth N` or `go movetime T`
  - `bestmove <move>`
- Use `stdin`/`stdout` loop.
- Log all UCI commands and responses.

---

## 11. Basic Time Management
**Goal:** Avoid flagging in timed games.  
**Tasks:**
- Track remaining time from `go` command.
- Allocate time per move (e.g., 1/30 of remaining time).

---

## 12. Playtesting & Debugging
**Goal:** Verify stability and correctness in real games.  
**Tasks:**
- Run Cody vs. itself at fixed depth.
- Run Cody vs. known engine at low depth.
- Watch for crashes, illegal moves, or hangs.
- Keep PGN logs of all games.

---

## 13. Incremental Improvements (Post‑MVP)
- Quiescence search
- Piece‑square tables
- Move ordering (killer moves, history heuristic)
- Transposition table
- Parallel search (thread pool integration)

---
