# Cody
# Rust Chess Engine with Fixed-Block Search Tree

## 📜 Overview
This project is a custom chess engine written in **Rust** with a singular initial goal:  
**Build the fastest possible exhaustive search framework** using a **memory-managed fixed block of nodes**.

The first milestone is not to create the strongest chess engine, but to establish a **deterministic, cache-friendly, and allocation-free search core** that can later be extended with more sophisticated evaluation and pruning techniques.

---

## 🧩 Core Concepts

### **Fixed Block Memory Model**
- All search nodes live in a **preallocated, fixed-size memory block**.
- Nodes are **recycled** after each turn to avoid heap allocations.
- Improves **cache locality** and prevents **heap fragmentation**.

### **Exhaustive Tree Expansion**
- Expands **every legal continuation** from the root to a fixed depth or node limit.
- Internal nodes are **not evaluated** — only leaves are scored.
- Stops when:
  - The **node budget** is reached, or
  - A **terminal game state** (checkmate, stalemate, insufficient material) is found.

### **Leaf-Only Evaluation**
- Minimal evaluation: **material balance only**.
- No positional heuristics in the initial version.
- Keeps evaluation cost negligible to focus on raw search speed.

### **Move Selection**
- Each leaf is tied to its **root branch**.
- Scores are aggregated per branch; the branch with the best aggregate wins.

### **Incremental Tree Reuse**
- After a move, only subtrees along the chosen line are kept.
- Invalid nodes are recycled into the fixed block.

---

## 🎯 Initial Development Goal
The **Phase 1** objective is to:
- Achieve **predictable runtime** and **deterministic memory usage**.
- Maximize **node throughput** per second.
- Validate the **fixed-block allocator** and **recycling logic** under load.

Strength, heuristics, and pruning are **deliberately deferred** until the search core is proven.

---

## 🚀 Future Work
Once the fast-search framework is stable, the following enhancements are planned:

### **Search Depth & Memory Tuning**
- Empirically determine optimal block size for depth vs. branching factor.
- Add instrumentation to track node usage and recycling efficiency.

### **Evaluation Improvements**
- Introduce lightweight positional heuristics at leaf nodes.
- Handle special cases like zugzwang, fortress draws, and tactical traps.

### **Search Efficiency**
- Experiment with **progressive deepening** while retaining fixed-block allocation.
- Add **transposition tables** to avoid redundant subtree evaluations.
- Consider minimal early cutoffs for forced mates or repetitions.

### **Move Aggregation Strategy**
- Test different aggregation methods (mean, max, weighted) to avoid bias.
- Define clear draw vs. advantage scoring rules.

### **Concurrency**
- Parallelize node expansion using Rust’s threading model.
- Avoid false sharing in the fixed block by using per-thread arenas.

### **Robustness**
- Add debug assertions to detect stale node reuse.
- Stress-test with real game positions, not just synthetic ones.

---

## 📌 Status
- **Current focus:** Framework for extremely fast, deterministic search.
- **Not yet implemented:** Positional evaluation, pruning, or adaptive depth.

---

## 🛠 License
MIT License — see `LICENSE` for details.
