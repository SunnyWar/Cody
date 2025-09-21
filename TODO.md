# Cody Chess Engine — MVP TODO Roadmap

## 0. Handle UCI Commands (Minimum for Arena Tournament)
- [x] **position**
  - Parse `startpos` or `fen`
  - Apply move list if present
- [x] **go**
  - Parse at least `movetime` and/or `wtime`/`btime`
  - Run search loop
  - Output `bestmove <move>`
- [x] **quit**
  - Exit cleanly

## 1. Search Core (Stub for MVP)
- [x] Implement fixed-depth or fixed-time search (depth=1 is enough for Arena to run games)
- [x] Generate legal moves from current position
- [x] Pick a move (random, eval-based, or fixed) and return via `bestmove`

## 2. Position Handling
- [x] FEN parser → internal board representation
- [x] Move parser (LAN/coordinate notation)
- [x] Apply moves to board state

## 3. Diagnostics & Traceability
- [x] Log all received UCI commands (for debugging)
- [x] Log all sent responses
- [x] Add `bench` command for internal testing (optional for Arena)

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
- [x] Implement `negamax(position, depth, alpha, beta) -> score`.
- [x] Terminal conditions:
  - Depth = 0 → return evaluation.
  - No legal moves → checkmate/stalemate scoring.
- [x] Add alpha‑beta pruning.
- [x] Count nodes, cutoffs, max depth reached.

---

## 6. Evaluation Function (Material‑Only MVP)
**Goal:** Assign a score to a position.  
**Tasks:**
- [x] Piece values: Pawn=100, Knight=320, Bishop=330, Rook=500, Queen=900.
- [x] Score = (material for side to move) − (material for opponent).
- [x] Log score breakdown by piece type. (basic evaluation implemented; advanced PST blending also present)

---

## 8. Iterative Deepening (Optional for MVP)
**Goal:** Improve move ordering and allow time control.  
**Tasks:**
- [ ] Stop when time runs out.

---

## 9. Incremental Improvements (Post‑MVP)
- [x] Quiescence search
- [x] Piece‑square tables
- [x] Move ordering (MVV/LVA implemented; killer/history heuristics pending)
- [ ] Transposition table
- [x] Parallel search (thread pool integration)

---
