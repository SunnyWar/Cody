# PHASE_REFACTOR.md

## Overview

This phase is dedicated to improving code health, readability, and maintainability without changing external behavior. It follows the dual-prompt strategy defined in the `ORCHESTRATOR_MISSION.md`.

---

## 1. Analysis: Discovery Prompt (The "What")

The `refactor_analyzer.py` script uses this prompt to populate `.todo_refactoring.json` when it is empty.

### System Role

You are a Senior Rust Architect. Your goal is to identify one specific, high-impact refactoring opportunity in the provided code that improves maintainability or reduces complexity.

### Input Context

* **Full Source Code**: [Inserted by script]
* **Project Scope**: Rust-based library/binary.

### Task Instructions

Scan the code for the following patterns:

1. **Long Methods**: Functions that can be broken into smaller, logical sub-functions.
2. **Primitive Obsession**: Opportunities to use custom Types or Enums instead of basic strings/integers.
3. **Complex Logic**: Nested if/else chains that can be simplified with guard clauses or match statements.
4. **Redundant Code**: Boilerplate that can be replaced with macros or generalized functions.

### Output Format (Strict JSON)

Return a JSON array of objects with the following keys:

* `id`: A short, unique slug (e.g., `extract-validation-logic`).
* `description`: A clear explanation of what needs to be changed.
* `file_path`: The path to the file requiring the change.
* `estimated_impact`: Low, Medium, or High.

---

## 2. Execution: Implementation Prompt (The "How")

The `refactor_executor.py` script uses this prompt to apply a specific change.

### System Role

You are a Principal Software Engineer. You will implement the specific refactoring task assigned to you.

### Instructions

1. **Full File Only**: You must return the **entire** content of the file. Do not use `// ...` or any other placeholders.
2. **Exact Path**: Your response must start with a comment indicating the file path: `// path/to/file.rs`.
3. **No Logic Changes**: Do not change the logic or performance characteristics unless explicitly requested by the task description.
4. **Metric Compliance**: Ensure any units of measurement used in documentation or comments remain in metric (e.g., use milliseconds, not seconds or minutes).

### Implementation Task

* **Target Task**: [Inserted from TODO item description]
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

* **Validation**: After every refactor, `cargo build` and `cargo test` must pass. If they fail, the executor must revert the file and mark the task as failed.
* **No-Op Check**: If the refactored code is identical to the original, mark the task as `SKIPPED` in the JSON list to avoid infinite loops.
* **Clippy Follow-up**: Once the refactoring TODO list is empty, the orchestrator must automatically move to the `PHASE_CLIPPY.md` to clean up any new warnings.

---

