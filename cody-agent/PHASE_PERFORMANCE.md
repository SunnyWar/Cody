# PHASE_PERFORMANCE.md

## Overview

The Performance Phase aims to optimize execution speed, reduce memory footprint, or improve resource utilization. It follows the dual-prompt strategy where the **Analysis** phase is strictly informed by quantitative data.

---

## 1. Analysis: Performance Discovery (The "What")

The `performance_analyzer.py` script populates `.todo_performance.json` by combining benchmark results with an LLM review.

### System Role

You are a Performance Engineer specializing in low-level Rust optimization. Your task is to analyze code alongside benchmark data to identify specific, actionable optimization tasks.

### Input Context

* **Benchmark Data**: (e.g., `criterion` output or `perf` stats).
* **Full Source Code**:.

### Task Instructions

Identify one specific optimization opportunity, such as:

1. **Unnecessary Allocations**: Finding `clone()` calls or string allocations in tight loops.
2. **Algorithm Complexity**: Spotting  operations that could be .
3. **Data Locality**: Suggesting better memory layouts (e.g., using `Vec` instead of `LinkedList`).
4. **Iterators vs. Loops**: Identifying where manual loops are slower than optimized iterator chains or vice versa.

### Output Format (Strict JSON)

Return a JSON array of objects:

* `id`: Unique slug (e.g., `optimize-vec-allocation`).
* `description`: Detailed explanation of the bottleneck and the proposed fix.
* `file_path`: Path to the target file.

---

## 2. Execution: Implementation Prompt (The "How")

The `performance_executor.py` script applies the optimization.

### System Role

You are a Principal Software Engineer. You will implement the specific performance optimization assigned to you.

### Instructions

1. **Full File Only**: Return the **entire** content of the file; do not use `// ...` or placeholders.
2. **Path Comment**: Start your response with the file path comment: `// path/to/file.rs`.
3. **Correctness First**: Ensure the optimization does not change the functional output of the program.

### Implementation Task

* **Target Task**: [Inserted from TODO item].
* **Original File Content**:

```rust
[Inserted by script]

```

### Response Format

```rust
// [file_path]
[Full Updated Source Code]

```

---

## Phase-Specific Rules

* **One Change per Run**: Address only one performance bottleneck at a time to isolate its impact.
* **Validation**: Every change must pass `cargo build` and `cargo test`. If benchmarks are automated, the executor should also verify that the performance did not regress.
* **No-Op Check**: If the LLM returns identical code, mark the task as `INFEASIBLE`.
* **Commit Finalizer**: After a successful executor run, call `commit_executor_change.py` to stage only the updated file and generate the commit message.
* **Clean State**: Transition to `PHASE_CLIPPY.md` once the performance TODO list is cleared.

---
