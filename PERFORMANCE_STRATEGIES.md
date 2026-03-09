# Performance Optimization Strategies

The `performance` phase now includes a comprehensive suite of **8 targeted optimization strategies** that the LLM can apply. Each iteration tries a different approach, allowing for diverse optimization exploration while keeping changes isolated and testable.

## Overview

The performance phase rotates through strategies to maximize the speed of the Cody chess engine while maintaining correctness. Each strategy focuses on a specific optimization technique, from function-level tuning to architecture-wide improvements.

## The 8 Strategies

### 1. **Single-Function Optimization**
**Focus:** Direct function speedup

Optimize a random function from a performance-critical file by eliminating:
- Unnecessary clones or copies
- Redundant computations  
- Inefficient field accesses
- Branch misprediction opportunities
- Cache-unfriendly memory access patterns

**Typical gains:** 2-10% on individual functions
**Examples:** Eliminating temporary allocations, hoisting repeated computations

---

### 2. **Bitboard Operation Optimization**
**Focus:** Hot path microoptimizations

Bitboard operations are called millions of times per search. Optimize by finding:
- Unnecessary intermediate bitboards that could be computed in one expression
- Multiple bitboard iterations that could be combined into a single pass
- Bitwise operations that could be reordered for better CPU pipeline utilization
- Masks that could be precomputed as thread-local or global constants

**Typical gains:** 5-15% (compound effect across millions of calls)
**Files:** `bitboard/src/movegen.rs`, `bitboard/src/attack.rs`, `bitboard/src/piecebitboards.rs`

---

### 3. **File-Level Analysis with Recommendation**
**Focus:** Strategic optimization planning

Analyze one complete performance-critical file to:
1. Identify the **3 most impactful optimization opportunities**
2. Rank them by estimated performance gain
3. Implement the **#1 ranked optimization** as a single change

**Typical gains:** 5-20% depending on the identified opportunities
**Approach:** Combines deep analysis with targeted execution

---

### 4. **Cache Locality Improvement**
**Focus:** Memory access efficiency

Optimize data layout and access patterns for better L1/L2 cache hits:
- Reorder struct fields to match access patterns
- Combine sequentially-accessed fields
- Reduce unnecessary pointer chasing in loops
- Align hot data to cache line boundaries

**Typical gains:** 3-8% (varies by CPU microarchitecture)
**Example:** Reordering `Position` fields to keep frequently-accessed data adjacent

---

### 5. **Hot Path Allocation Reduction**
**Focus:** Eliminating heap allocations from critical paths

Cody's fixed-block allocator requires zero heap allocations in hot loops. Find and eliminate:
- `Vec` allocations in search/move generation
- `Box` allocations in frequently-called functions
- `String` allocations in performance-critical paths
- Any heap allocation in move generation

Replace with:
- Stack-based arrays or small fixed structures
- Arena-allocated storage via the fixed-block allocator
- Preallocated thread-local buffers

**Typical gains:** 2-5% (variable based on allocation frequency)
**Philosophy:** Maintain the allocation-free constraint

---

### 6. **Branching and Prediction Optimization**
**Focus:** CPU pipeline efficiency

Reduce branch misprediction stalls by:
- Replacing unpredictable branches with bitwise operations (branchless code)
- Reordering conditions for better prediction patterns
- Combining multiple branches into single computed paths
- Early termination in ways that boost prediction

**Typical gains:** 1-4% per optimization
**Example:** Replace `if-else` with `&` and `|` operations in tight loops

---

### 7. **Hot Path Dataflow Simplification**
**Focus:** Reducing redundant work in critical call chains

Simplify hot-path dataflow without relying on forced inlining:
- Fold repeated index/mask/address computations into single computed values per iteration
- Replace copy-modify-writeback patterns with direct in-place mutation where safe
- Collapse thin wrappers into direct call paths only when it reduces duplicated work

**Typical gains:** 1-5% per function
**Caution:** Preserve readability and verify correctness with full test/bench runs

---

### 8. **Loop Optimization and Iteration**
**Focus:** Iteration efficiency

Optimize loops by:
- Precomputing loop bounds instead of recalculating each iteration
- Hoisting invariant computations outside loops
- Combining multiple passes over the same data (loop fusion)
- Unrolling critical innermost loops
- Replacing iterator patterns with direct index-based loops where beneficial

**Typical gains:** 2-8% depending on loop characteristics
**Example:** Combine adjacent loops over move lists into a single pass

---

## Performance-Critical Files

The strategies target these files (in order of impact on engine speed):

| Priority | File | Why | Calls Per Search |
|----------|------|-----|-------------------|
| 🔴 1 | `bitboard/src/movegen.rs` | Move generation core | ~1,000,000+ |
| 🔴 2 | `bitboard/src/position.rs` | Apply move operation | ~1,000,000+ |
| 🟠 3 | `bitboard/src/attack.rs` | Square attack detection | ~500,000+ |
| 🟠 4 | `engine/src/search/engine.rs` | Main search loop | ~100,000+ |
| 🟡 5 | `engine/src/core/arena.rs` | Arena allocation patterns | ~1,000,000+ |
| 🟡 6 | `bitboard/src/piecebitboards.rs` | Piece board operations | ~500,000+ |

---

## How It Works

Each time the `performance` phase runs:

```
Iteration 1 → Strategy 1 (single-function)
Iteration 2 → Strategy 2 (bitboard optimization)
Iteration 3 → Strategy 3 (file-level analysis)
...
Iteration 8 → Strategy 8 (loop optimization)
```

For each iteration:

1. **LLM Analysis:** The system provides the strategy and a random target file/function
2. **Optimization:** The LLM generates a unified diff implementing the strategy
3. **Validation:** The change is applied, built, tested, and benchmarked
4. **Decision:** If successful, it's kept; if it fails, it's reverted

---

## Integration Example

When running the full orchestration:

```bash
# Run all phases including performance
python .\cody-graph\main.py all

# Or run only performance phase (iterates through all 8 strategies)
python .\cody-graph\main.py performance
```

The performance phase will:
- Automatically cycle through all 8 strategies
- Skip failed optimizations and continue to the next strategy
- Build and test after each successful change
- Stop after all strategies have been attempted

---

## Architecture Constraints

All optimizations **must preserve**:

✅ **Allocation-Free Hot Paths:** No new heap allocations in search/movegen
✅ **Correctness:** All changes must pass perft tests and play correctly
✅ **APIs:** Public interfaces remain unchanged
✅ **Determinism:** Engine outputs remain the same for same positions

---

## Expected Performance Gains

Based on the strategy suite:

| Best Case | Expected | Conservative |
|-----------|----------|--------------|
| **40-60%** combined gain | **10-20%** combined | **3-5%** minimum |
| (All strategies successful) | (5-6 strategies succeed) | (2-3 strategies succeed) |

Individual strategy gains typically range from **1-15%** depending on the code being optimized.

---

## Common Patterns the Strategies Target

### Redundant Computation
```rust
// Before (costs 2 popcount operations)
let count1 = bishops.popcount();
let count2 = bishops.popcount();

// After (costs 1)
let count = bishops.popcount();
```

### Unnecessary Intermediate Values
```rust
// Before
let temp = a & b;
let result = temp | c;

// After
let result = (a & b) | c;
```

### Branchless Operations
```rust
// Before (branch misprediction risk)
let score = if piece_count > 8 { endgame_eval() } else { midgame_eval() };

// After (branchless)
let score = if_else_branchless(piece_count > 8, endgame_eval(), midgame_eval());
```

### Memory Layout Optimization
```rust
// Before: Fields accessed in random order
struct Position {
    occupancy: u64,      // Line 1
    piece_counts: [u8],  // Line 2 (different line)
    occupancy_again: u64,// Back to line 1
    ...
}

// After: Frequently-accessed fields grouped
struct Position {
    occupancy: u64,
    piece_counts: [u8],
    castling_rights: u8, // Still line 1
    ...
}
```

---

## Monitoring Progress

Monitor the performance phase with:

```powershell
# Check orchestrator state after running
cat orchestrator_state.json | jq '.phases_completed'

# See which strategies succeeded
grep "Applying strategy" engine_output.txt
```

---

## Future Enhancements

Potential additional strategies for future iterations:

- **Evaluation tuning:** Adjust eval weights for speed
- **Search parameter tuning:** Optimal depth/breadth trades
- **SIMD optimization:** Vector operations for bitboards
- **Parallel search:** Multi-threaded improvements
- **Transposition table optimization:** Hash function tuning

---

## Philosophy

The performance suite follows these principles:

1. **Diversity:** Each strategy targets different optimization angles
2. **Isolation:** One change per iteration, easy to revert if needed
3. **Validation:** Build + test after every change
4. **Safety:** Conservative estimates; only keep proven improvements
5. **Simplicity:** Avoid complex refactorings; focus on speed

This approach maximizes the chances of finding compounding improvements while maintaining code stability.
