# Cody Chess Engine — MVP TODO Roadmap

## 0. Handle UCI Commands (Minimum for Arena Tournament)
- [ ] **position**
  - Parse `startpos` or `fen`
  - Apply move list if present
- [ ] **go**
  - Parse at least `movetime` and/or `wtime`/`btime`
  - Run search loop
  - Output `bestmove <move>`
- [ ] **quit**
  - Exit cleanly

## 1. Search Core (Stub for MVP)
- [ ] Implement fixed-depth or fixed-time search (depth=1 is enough for Arena to run games)
- [ ] Generate legal moves from current position
- [ ] Pick a move (random, eval-based, or fixed) and return via `bestmove`

## 2. Position Handling
- [ ] FEN parser → internal board representation
- [ ] Move parser (LAN/coordinate notation)
- [ ] Apply moves to board state

## 3. Diagnostics & Traceability
- [ ] Log all received UCI commands (for debugging)
- [ ] Log all sent responses
- [ ] Add `bench` command for internal testing (optional for Arena)

## 4. Post-MVP (Future Work)
- [ ] Implement iterative deepening, alpha-beta, transposition table
- [ ] Add `setoption` handling
- [ ] Add ponder support
- [ ] Time management improvements
- [ ] UCI `stop` command handling
- [ ] Optimize with PGO
---

## 5. Search Core (Negamax + Alpha‑Beta)
**Goal:** Basic minimax search with pruning.  
**Tasks:**
- Implement `negamax(position, depth, alpha, beta) -> score`.
- Terminal conditions:
  - Depth = 0 → return evaluation.
  - No legal moves → checkmate/stalemate scoring.
- Add alpha‑beta pruning.
- Count nodes, cutoffs, max depth reached.

---

## 6. Evaluation Function (Material‑Only MVP)
**Goal:** Assign a score to a position.  
**Tasks:**
- Piece values: Pawn=100, Knight=320, Bishop=330, Rook=500, Queen=900.
- Score = (material for side to move) − (material for opponent).
- Log score breakdown by piece type.

---

## 8. Iterative Deepening (Optional for MVP)
**Goal:** Improve move ordering and allow time control.  
**Tasks:**
- Search depth 1, then 2, etc., keeping best move so far.
- Stop when time runs out.

---

## 9. Incremental Improvements (Post‑MVP)
- Quiescence search
- Piece‑square tables
- Move ordering (killer moves, history heuristic)
- Transposition table
- Parallel search (thread pool integration)

---
