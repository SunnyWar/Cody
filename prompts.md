# Cody Prompts

## Project Intent (from README)

Cody’s mission is an AI-driven improvement loop with minimal human intervention.

For this agent, the implementation scope is **only**:

- orchestration reliability,
- state-machine correctness,
- Python automation scripts,
- diagnostics and observability.

This means fixes should target:

- `cody-graph/main.py`
- `cody-graph/agents/*.py`
- `cody-graph/graph/*.py`
- `cody-graph/state/*.py`
- `cody-graph/tools/*.py`
- orchestration config/state files when required

And should **not** directly fix Rust engine/chess logic in:

- `bitboard/`
- `engine/`

unless explicitly instructed in a separate request.

---

## Reusable Prompt: Fix Failed Orchestration Run from Logs

Use this prompt when providing a failed run log to repair the orchestrator.

```text
You are GitHub Copilot (GPT-5.3-Codex) operating in the Cody repository.

Critical scope boundary:
- Your job is ONLY to fix orchestration and Python automation.
- Do NOT modify Rust chess engine code (`bitboard/`, `engine/`) unless explicitly told.

Primary objective:
- Make the state machine and each orchestration phase run correctly end-to-end.
- Improve reliability, debuggability, and deterministic behavior of the automation loop.
- Add diagnostic logging where needed to make failures actionable.

Allowed areas:
- cody-graph/main.py
- cody-graph/agents/*.py
- cody-graph/graph/*.py
- cody-graph/state/*.py
- cody-graph/tools/*.py
- cody-agent/config.json and orchestration state/diagnostic outputs when necessary

Task:
I am providing output from a failed run. Analyze it, identify root causes in the orchestrator/state machine/python scripts, implement fixes, and verify.

Failure log:
=== BEGIN FAILED RUN LOG ===
{{PASTE_LOG_HERE}}
=== END FAILED RUN LOG ===

Required workflow:
1) Classify failures (phase transition, state persistence, tool invocation, subprocess handling, config loading, path/env, rollback/apply-diff, timeout/retry).
2) Locate exact failing modules/functions in cody-graph/cody-agent Python code.
3) Implement minimal root-cause fixes.
4) Add/upgrade diagnostics logs around decision points and failure boundaries (phase entry/exit, command execution, diff apply, verification gates, rollback).
5) Re-run targeted validation to confirm the failing path now completes.
6) Continue until the orchestrator run is clean or you hit a true blocker.
7) Summarize: root cause, files changed, diagnostics added, verification performed, remaining risk.

Acceptance criteria:
- Failed orchestration path is fixed.
- State transitions are correct and observable in logs.
- Diagnostics are sufficient to debug the next failure quickly.
- No unrelated changes.
- No Rust engine/chess logic edits.
```

---

## Optional Variant: Diagnostics-First Pass

```text
Use the same prompt, but add:
- Before changing behavior, first add precise logging/diagnostics to isolate root cause.
- Then apply the smallest behavior fix needed.
```
