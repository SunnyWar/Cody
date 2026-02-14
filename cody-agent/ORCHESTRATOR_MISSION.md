# Orchestrator Mission

## Core Mission
The orchestrator runs automated code improvement cycles. Each run follows a "single-task" constraint to ensure stability and traceability:

1. Identify: Find one improvement opportunity via deterministic tools or an analysis LLM.
2. Implement: Call an execution LLM to generate the fix.
3. Apply: Overwrite the target file with the new content.
4. Validate: Verify changes with `cargo build` and `cargo test`.
5. Commit: Run the commit finalizer script to generate a message and stage only the executor-updated file.
6. Exit: Terminate the process so the next run starts fresh.

## Split-Prompt Architecture
Each phase (refactoring, performance, features, clippy) is governed by its own dedicated document. These documents define two distinct prompt types.

### Analysis Prompts (the "what")
- Purpose: Scan the codebase to identify specific, actionable tasks.
- Trigger: Run by the `_analyzer.py` script when the `.todo_<category>.json` is empty or missing.
- Clippy exception: Clippy does not use an LLM for analysis; it uses `cargo clippy --json` output directly.
- Output: A structured list of TODO items saved to JSON.

### Execution Prompts (the "how")
- Purpose: Direct the LLM to implement a specific TODO item.
- Constraint: Must always include the full file content and require a full file return.
- Reference: These prompts are stored in the phase-specific satellite documents (for example, `PHASE_REFACTOR.md`).

## Standard Executor Pattern
All executors (refactoring, performance, features, clippy) follow this pattern.

1. Pre-execution validation: Run `validate_cargo.py` before the phase starts. If it fails, abort with the error message: "Aborting: Base code is broken. Fix existing errors before running automated improvements."
2. Workspace sanity check: Verify `git status` is clean before proceeding.
3. Analyzer phase: Run analysis tools, generate TODO items with structured metadata, save to `.todo_<category>.json`.
4. Executor phase: Load the next TODO item, gather full file context, call the LLM, overwrite files with full-file output, and validate with `cargo build` and `cargo test`.
5. Commit finalizer: Record the last changed file in `.last_executor_change.json` and run `commit_executor_change.py` to stage only that file and generate a commit message.
6. Orchestrator integration: Execute a single task, commit on success, and exit so the next run can continue.

## Phase Flow and Delegation
The orchestrator moves through phases in a strict linear sequence. After any successful refactoring, performance, or feature task, the Clippy phase is re-triggered automatically to keep the codebase clean. For detailed prompt engineering and phase-specific rules, refer to the satellite files.

| Phase       | Analysis Method                 | Satellite Document |
|-------------|---------------------------------|--------------------|
| Refactoring | LLM discovery prompt            | `PHASE_REFACTOR.md`|
| Clippy      | Deterministic (`cargo clippy`)  | `PHASE_CLIPPY.md`  |
| Performance | LLM discovery + benchmarks      | `PHASE_PERFORMANCE.md` |
| Features    | LLM discovery / user input      | `PHASE_FEATURE.md` |

## LLM Implementation Instructions (for code generation)
When generating or fixing the Python scripts (`orchestrator.py`, `executor.py`, etc.), the following rules are non-negotiable:

- Idempotency: `orchestrator.py` must toggle `analysis_done` to `False` whenever the main phase advances so the next category's analyzer runs.
- Anti-lazy coding: `executor.py` must explicitly forbid ellipses (`// ...`) or placeholders in its prompt templates.
- Change verification: If the LLM returns code identical to the source, the executor must mark the task as `SKIPPED` or `INFEASIBLE` to prevent infinite loops.
- Path mapping: Use regex to extract code blocks strictly following the `// path/to/file.rs` header format.
- Validation rigidity: Treat any non-zero exit code from cargo as a failure; revert changes immediately using `git checkout -- <file>` or `git restore <file>` in `executor.py`.
- No LLM in orchestrator logic: The core orchestrator logic must remain deterministic; LLM calls are reserved for the executor and analyzer modules.

## Common Mistakes to Avoid
- Merging tasks: Never attempt to fix two TODOs in one run.
- Snippet updates: Never accept partial code; always process full files.
- State stagnation: Do not forget to reset phase tracking in `orchestrator_state.json`.
- Blind commits: Do not commit code that failed `cargo test`.
- Hardcoded prompts: Load prompts from the phase-specific `.md` files or specialized templates, not in `orchestrator.py`.