# PHASE_CLIPPY.md

## Overview

This phase handles the automated fixing of Rust lints and compiler warnings. Unlike Refactoring or Performance phases, the **Analysis** step is deterministic and does not require an LLM to identify tasks.

---

## 1. Analysis: Deterministic Parsing (The "What")

The `clippy_analyzer.py` script identifies tasks by running the compiler.

### Process

1. **Command**: Execute `cargo clippy --message-format=json`.
2. **Stream filter (clippy_parser.py)**: Use `clippy_parser.py` to filter the JSON stream, keeping only actual `clippy::` diagnostics with levels "warning" or "error".
3. **Deduplicate**: Group by diagnostic code (e.g., `clippy::needless_return`) and file path.
4. **Ingestion limit**: `clippy_analyzer.py` must ingest only a capped sample (for example, the first 50 warnings after deduplication) from the `clippy_parser.py` stream so `.todo_clippy.json` stays LLM-friendly.
5. **Output**: Populate `.todo_clippy.json` with structured items.

### TODO Item Structure

* `id`: Unique identifier (e.g., `clippy-file-line-code`).
* `message`: The specific suggestion or warning text provided by Clippy.
* `file_path`: Path to the affected `.rs` file.
* `line_number`: The line where the issue was detected.
* `code_snippet`: The specific code triggering the warning.

---

## 2. Execution: Implementation Prompt (The "How")

The `clippy_executor.py` uses this prompt to direct the LLM to fix the specific diagnostic.

### System Role

You are an expert Rust Developer. Your task is to resolve a specific Clippy warning or error while maintaining the exact logic of the original code.

### Instructions

1. **Full File Only**: You must return the **entire** content of the file. Do not use `// ...` or placeholders.
2. **Exact Path**: Your response must start with a comment indicating the file path: `// path/to/file.rs`.
3. **Minimal Change**: Fix only the specific issue described in the Clippy diagnostic. Do not perform unrelated refactoring.
4. **Single-warning focus**: You are being shown one specific warning among many. Do not attempt to fix other warnings in the file; focus exclusively on the provided diagnostic.


### Implementation Task

* **Clippy Diagnostic**: [Inserted from TODO: "warning: needless return..."]
* **Affected File**: [Inserted path/to/file.rs]
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

* **One Warning at a Time**: The executor must only address the single TODO item provided in the prompt to ensure the "One task per run" rule is followed.
* **Verification**: After applying the fix, the orchestrator must run `cargo clippy` again to ensure the specific warning is gone and `cargo test` to ensure no regressions.
* **Persistence rule**: If the warning persists after an LLM fix, mark the task as `FAILED` or `BLOCKED` to prevent looping on unfixable lints.
* **No-Op Check**: If the LLM returns the exact same file content, mark the task as `INFEASIBLE` and exit to prevent loops.
* **Commit Finalizer**: After a successful executor run, call `commit_executor_change.py` to stage only the updated file and generate the commit message.

---
