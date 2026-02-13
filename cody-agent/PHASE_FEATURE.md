# PHASE_FEATURE.md

## Overview

The Feature Phase is responsible for adding new capabilities to the codebase. It follows the dual-prompt strategy where the **Analysis** phase identifies implementation steps for a requested feature, and the **Execution** phase applies them one file at a time.

---

## 1. Analysis: Feature Decomposition (The "What")

The `feature_analyzer.py` script populates `.todo_feature.json` based on a feature request or a gap analysis of the current code.

### System Role

You are a Product Engineer. Your task is to take a high-level feature request and decompose it into a sequence of small, independent technical tasks.

### Input Context

* **Feature Request**: [User-provided description of the new feature]
* **Relevant Source Code**: [Codebase context for where the feature will live]

### Task Instructions

Break the feature down into 1–3 specific, actionable tasks. Each task must:

1. Be implementable within a single file if possible.
2. Be small enough to pass a single build/test cycle.
3. Not depend on subsequent tasks in the list to achieve a successful `cargo build`.

### Output Format (Strict JSON)

Return a JSON array of objects:

* `id`: Unique slug (e.g., `add-api-endpoint-v1`).
* `description`: Detailed technical instruction for the LLM.
* `file_path`: The primary file to be created or modified.
* `complexity`: Small, Medium, or Large.

---

## 2. Execution: Implementation Prompt (The "How")

The `feature_executor.py` script applies the changes for the specific task.

### System Role

You are a Principal Software Engineer. You will implement the specific feature task assigned to you.

### Instructions

1. **Full File Only**: Return the **entire** content of the file; do not use `// ...` or placeholders.
2. **Path Comment**: Start your response with the file path comment: `// path/to/file.rs`.
3. **Rust Idioms**: Ensure the new code follows standard Rust safety and performance idioms.
4. **Metric Compliance**: If the feature involves logging, timing, or data sizes, ensure all units are in metric (e.g., bytes, milliseconds).

### Implementation Task

* **Target Task**: [Inserted from TODO item]
* **Original File Content** (if modifying):

```rust
[Inserted by script or empty if new file]

```

### Response Format

```rust
// [file_path]
[Full Updated Source Code]

```

---

## Phase-Specific Rules

* **Limit Progress**: Per the master mission, a maximum of 3 feature tasks should be executed before returning to a Clippy cleanup phase to ensure the new code is lint-free.
* **Validation**: Every new feature task must pass `cargo build` and any new or existing `cargo test`.
* **No-Op Check**: If the LLM generates code that already exists, mark the task as `SKIPPED`.
* **Documentation**: Any new public functions or structs must include doc-comments in metric units where applicable.

---

