# Cody
# Rust Chess Engine with Fixed‑Block Search Tree

## 📜 Overview
Cody is a custom chess engine written in **Rust** with a singular initial goal:  
**Build the fastest possible exhaustive search framework** using a **memory‑managed fixed block of nodes**, backed by a fully optimised **pseudo‑legal move generator**.

The first milestone is not to create the strongest chess engine, but to establish a **deterministic, cache‑friendly, allocation‑free search core** with a **complete, efficient move generation layer** that can later be extended with more sophisticated evaluation and pruning techniques.

---

## 🧩 Core Concepts

### **Fixed Block Memory Model**
- All search nodes live in a **preallocated, fixed‑size memory block**.
- Nodes are **recycled** after each turn to avoid heap allocations.
- Improves **cache locality** and prevents **heap fragmentation**.

### **Optimised Pseudo‑Legal Move Generation**
- Per‑piece generators for pawns, knights, bishops, rooks, queens, and kings.
- Generates **all possible moves** (captures + quiet moves) without legality filtering.
- Uses a shared **`MoveGenContext`** to avoid recomputing `occ` and `not_ours` in each generator.
- Employs **geometry pre‑masking**, **early‑bail**, and **zero‑check** patterns for speed.
- Includes **debug geometry assertions** to validate attack tables in development builds.
- Fully type‑safe with explicit `u8` casts for `Move::new`.

### **Exhaustive Tree Expansion**
- Expands every pseudo‑legal continuation from the root to a fixed depth or node limit.
- Internal nodes are not evaluated — only leaves are scored.
- Stops when:
  - The node budget is reached, or
  - A terminal game state (checkmate, stalemate, insufficient material) is found.

### **Leaf‑Only Evaluation**
- Minimal evaluation: material balance only.
- No positional heuristics in the initial version.
- Keeps evaluation cost negligible to focus on raw search speed.

### **Move Selection**
- Each leaf is tied to its root branch.
- Scores are aggregated per branch; the branch with the best aggregate wins.

### **Incremental Tree Reuse**
- After a move, only subtrees along the chosen line are kept.
- Invalid nodes are recycled into the fixed block.

---

## 🎯 Initial Development Goal
The **Phase 1** objective is to:
- Achieve **predictable runtime** and **deterministic memory usage**.
- Maximise **node throughput** per second.
- Validate the **fixed‑block allocator** and **recycling logic** under load.
- Ensure **complete and efficient pseudo‑legal move generation** for all pieces.

Strength, heuristics, and pruning are **deliberately deferred** until the search core and movegen are proven.

---

## 🚀 Future Work

### **Search Depth & Memory Tuning**
- Empirically determine optimal block size for depth vs. branching factor.
- Add instrumentation to track node usage and recycling efficiency.

### **Evaluation Improvements**
- Introduce lightweight positional heuristics at leaf nodes.
- Handle special cases like zugzwang, fortress draws, and tactical traps.

### **Search Efficiency**
- Experiment with **progressive deepening** while retaining fixed‑block allocation.
- Add **transposition tables** to avoid redundant subtree evaluations.
- Consider minimal early cutoffs for forced mates or repetitions.

### **Move Aggregation Strategy**
- Test different aggregation methods (mean, max, weighted) to avoid bias.
- Define clear draw vs. advantage scoring rules.

### **Concurrency**
- Parallelise node expansion using Rust’s threading model.
- Avoid false sharing in the fixed block by using per‑thread arenas.

### **Robustness**
- Add debug assertions to detect stale node reuse.
- Stress‑test with real game positions, not just synthetic ones.

---

## 📌 Status
- **Current focus:** Extremely fast, deterministic search with a complete pseudo‑legal move generator.
- **Implemented:** Fixed‑block allocator, per‑piece movegen with context‑based optimisation, debug geometry checks.
- **Not yet implemented:** Positional evaluation, pruning, adaptive depth, legality filtering.

---

## 🛠 License
MIT License — see `LICENSE` for details.
