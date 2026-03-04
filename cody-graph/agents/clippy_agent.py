import json
import os
import subprocess
from pathlib import Path
from typing import Optional, Tuple

from openai import OpenAI

from state.cody_state import CodyState

DEFAULT_MAX_PHASE_ITERATIONS = 8

def _load_config(repo_path: str) -> dict:
    config_override = os.environ.get("CODY_CONFIG_PATH")
    if config_override:
        config_path = Path(config_override)
    else:
        config_path = Path(repo_path) / "cody-agent" / "config.json"
    if not config_path.exists():
        return {}
    try:
        return json.loads(config_path.read_text(encoding="utf-8"))
    except Exception:
        return {}

def _select_model(config: dict, phase: str) -> str:
    """Select the appropriate model for the given phase."""
    models = config.get("models", {}) if isinstance(config, dict) else {}
    # Try phase-specific model first, fall back to clippy, then config default, then hardcoded default
    return models.get(phase) or models.get("clippy") or config.get("model") or "gpt-4o-mini"

def _extract_all_issues(output: str) -> Tuple[str, Optional[str], Optional[int]]:
    """Extract all clippy warnings/errors for agent visibility.
    
    Returns: (all_issues_text, first_file_path, first_line_no)
    """
    lines = output.splitlines()
    issues: list[str] = []
    first_file: Optional[str] = None
    first_line: Optional[int] = None
    
    i = 0
    while i < len(lines):
        stripped = lines[i].lstrip()
        if stripped.startswith("warning:") or stripped.startswith("error:"):
            issue_block: list[str] = []
            start_i = i
            issue_file: Optional[str] = None
            issue_line: Optional[int] = None
            
            # Collect this issue until we hit the next one
            while i < len(lines):
                line = lines[i]
                stripped_inner = line.lstrip()
                if i > start_i and (stripped_inner.startswith("warning:") or stripped_inner.startswith("error:")):
                    break
                issue_block.append(line)
                
                # Extract file/line for each issue
                if "-->" in line:
                    arrow = line.split("-->", 1)[1].strip()
                    parts = arrow.rsplit(":", 2)
                    if len(parts) == 3:
                        issue_file = parts[0].strip()
                        try:
                            issue_line = int(parts[1])
                        except ValueError:
                            pass
                
                # Set first_file/first_line from first issue
                if first_file is None and issue_file:
                    first_file = issue_file
                    first_line = issue_line
                
                i += 1
            
            # Store issue with metadata for filtering
            issues.append({
                "text": "\n".join(issue_block).strip(),
                "file": issue_file,
                "line": issue_line,
                "signature": f"{issue_file}:{issue_line}:{lines[start_i].strip()[:50]}" if issue_file and issue_line else None
            })
        else:
            i += 1
    
    if not issues:
        return (output.strip(), None, None)
    
    return (issues, first_file, first_line)

def _filter_attempted_issues(issues: list[dict], attempted: list[str]) -> list[dict]:
    """Filter out issues that have already been attempted."""
    if not attempted:
        return issues
    
    filtered = []
    for issue in issues:
        sig = issue.get("signature")
        # Only include issues that have a valid signature and haven't been attempted
        if sig and sig not in attempted:
            filtered.append(issue)
        elif not sig:
            # Issue without signature - include it but log warning
            print(f"[cody-graph] [DIAG] Warning: Issue without signature, including anyway", flush=True)
            filtered.append(issue)
    
    return filtered

def _read_context_snippet(repo_path: str, file_path: Optional[str], line_no: Optional[int], radius: int = 20) -> str:
    if not file_path or not line_no:
        return ""

    full_path = file_path
    if not os.path.isabs(full_path):
        full_path = os.path.join(repo_path, file_path)

    if not os.path.exists(full_path):
        return ""

    try:
        lines = Path(full_path).read_text(encoding="utf-8").splitlines()
    except Exception:
        return ""

    start = max(1, line_no - radius)
    end = min(len(lines), line_no + radius)
    snippet_lines = []
    for i in range(start, end + 1):
        snippet_lines.append(f"{i:4d} | {lines[i - 1]}")

    try:
        rel_path = os.path.relpath(full_path, repo_path)
    except ValueError:
        rel_path = full_path

    return "File: " + rel_path + "\n" + "\n".join(snippet_lines)

def _read_file_head_snippet(repo_path: str, file_path: str, max_lines: int = 120) -> str:
    full_path = file_path
    if not os.path.isabs(full_path):
        full_path = os.path.join(repo_path, file_path)
    if not os.path.exists(full_path):
        return ""
    try:
        lines = Path(full_path).read_text(encoding="utf-8").splitlines()
    except Exception:
        return ""

    shown = lines[:max_lines]
    numbered = [f"{i+1:4d} | {line}" for i, line in enumerate(shown)]
    try:
        rel_path = os.path.relpath(full_path, repo_path)
    except ValueError:
        rel_path = full_path
    return f"File: {rel_path}\n" + "\n".join(numbered)

def _get_system_prompt_for_phase(phase: str) -> str:
    """Get the system prompt appropriate for the current phase."""
    phase_prompts = {
        "clippy": """
You are Cody's ClippyAgent. 
Goal: Reduce Clippy warnings and fix compilation errors in this Rust project.

CONTEXT PROVIDED:
1. Source Code: You will see the content of .rs files.
2. Clippy/Compiler Output: You will see the current warnings/errors.

STRICT RULES:
- Only suggest changes to existing files in the repo.
- Respond with a short explanation of the fix.
- Provide the fix as a UNIFIED DIFF in a markdown diff block.
- REQUIRED FORMAT EXAMPLE:
    --- a/engine/src/search/core.rs
    +++ b/engine/src/search/core.rs
    @@ -157,1 +157,1 @@
    -    let mut moves_vec = moves;
    +    let moves_vec = moves;
- The @@ hunk header MUST include actual line numbers like: @@ -157,1 +157,1 @@
- NEVER use @@ @@ without numbers - this is INVALID
- Do NOT use *** markers or *** Begin Patch / *** End Patch
- Put your diff inside a markdown code block with 'diff' language tag
- Do not suggest external dependencies.
- CRITICAL: Fix EXACTLY ONE warning/error at a time with minimal changes.
- CRITICAL: When multiple issues exist, ALWAYS pick the SIMPLEST fix (fewest lines changed).
- CRITICAL: NEVER add #[allow(...)], #[warn(...)], or any suppression attributes.
- ALWAYS fix the root cause. Suppressing warnings/errors is forbidden.
- Skip issues that require large refactors (>20 lines); pick simpler ones first.
""",
        "refactoring": """
You are Cody's RefactoringAgent.
Goal: Improve code quality and maintainability through refactoring.

CONTEXT PROVIDED:
1. Source Code: You will see the content of .rs files.
2. Analysis: You will see refactoring opportunities identified.

STRICT RULES:
- Only refactor without changing behavior.
- Respond with a short explanation of the refactoring.
- Provide changes as a UNIFIED DIFF in a markdown diff block.
- REQUIRED FORMAT EXAMPLE:
    --- a/engine/src/search/core.rs
    +++ b/engine/src/search/core.rs
    @@ -157,1 +157,1 @@
    -    let mut moves_vec = moves;
    +    let moves_vec = moves;
- The @@ hunk header MUST include actual line numbers like: @@ -157,1 +157,1 @@
- NEVER use @@ @@ without numbers - this is INVALID
- Do NOT use *** markers
- Put your diff inside a markdown code block with 'diff' language tag
- NEVER add #[allow(...)], #[warn(...)], or any suppression attributes.
- Maintain architecture constraints (allocation-free hot path, fixed-block arena).
""",
        "performance": """
You are Cody's PerformanceAgent.
Goal: Optimize for speed and efficiency while maintaining correctness.

CONTEXT PROVIDED:
1. Source Code: Relevant Rust implementation.
2. Benchmarks: Current performance metrics.

STRICT RULES:
- Only optimize, do not refactor unnecessarily.
- Provide changes as a UNIFIED DIFF in a markdown diff block.
- REQUIRED FORMAT EXAMPLE:
    --- a/engine/src/search/core.rs
    +++ b/engine/src/search/core.rs
    @@ -157,1 +157,1 @@
    -    let mut moves_vec = moves;
    +    let moves_vec = moves;
- The @@ hunk header MUST include actual line numbers like: @@ -157,1 +157,1 @@
- NEVER use @@ @@ without numbers - this is INVALID
- Do NOT use *** markers
- Put your diff inside a markdown code block with 'diff' language tag
- NEVER add #[allow(...)], #[warn(...)], or any suppression attributes.
- Target ≥5% performance improvement.
- Test for correctness with perft and benchmarks.
""",
        "features": """
You are Cody's FeatureAgent.
Goal: Implement new chess engine capabilities and features.

CONTEXT PROVIDED:
1. Architecture: Design patterns and constraints.
2. Requirements: Feature specification.

STRICT RULES:
- Follow the fixed-block arena allocation model.
- Provide changes as a UNIFIED DIFF in a markdown diff block.
- REQUIRED FORMAT EXAMPLE:
    --- a/engine/src/search/core.rs
    +++ b/engine/src/search/core.rs
    @@ -157,1 +157,1 @@
    -    let mut moves_vec = moves;
    +    let moves_vec = moves;
- The @@ hunk header MUST include actual line numbers like: @@ -157,1 +157,1 @@
- NEVER use @@ @@ without numbers - this is INVALID
- Do NOT use *** markers
- Put your diff inside a markdown code block with 'diff' language tag
- NEVER add #[allow(...)], #[warn(...)], or any suppression attributes.
- Each feature should pass all tests.
""",
        "ELOGain": """
You are Cody's ELOGainAgent.
Goal: Find and implement improvements specifically to increase the chess engine's ELO rating.

CONTEXT PROVIDED:
1. Source Code: Current engine implementation.
2. Architecture: Cody uses a fixed-block arena, pseudo-legal move generation, and allocation-free hot paths.
3. Performance Data: Search depth, position evaluation metrics.

STRICT RULES:
- Focus ONLY on changes that will improve playing strength (ELO).
- Target areas: evaluation improvements, search enhancements (pruning, extensions), move ordering.
- Provide changes as a UNIFIED DIFF in a markdown diff block.
- REQUIRED FORMAT EXAMPLE:
    --- a/engine/src/search/evaluator.rs
    +++ b/engine/src/search/evaluator.rs
    @@ -42,1 +42,3 @@
    -    score
    +    let endgame_factor = calculate_endgame_factor(pos);
    +    score * (1.0 + endgame_factor * 0.1)
- The @@ hunk header MUST include actual line numbers like: @@ -42,1 +42,3 @@
- NEVER use @@ @@ without numbers - this is INVALID
- Do NOT use *** markers
- Put your diff inside a markdown code block with 'diff' language tag
- NEVER add #[allow(...)], #[warn(...)], or any suppression attributes.
- Each change must maintain correctness (pass perft and tests).
- Prioritize changes with measurable ELO impact (eval tuning, search improvements).
""",
    }
    return phase_prompts.get(phase, phase_prompts["clippy"])

def clippy_agent(state: CodyState) -> CodyState:
    from datetime import datetime
    
    phase = state.get("current_phase", "clippy")
    print(f"[cody-graph] clippy_agent: START (phase: {phase})", flush=True)
    api_key = os.environ.get("OPENAI_API_KEY")
    if not api_key:
        result_state = {
            **state,
            "last_command": "clippy_llm_think",
            "last_output": "Missing OPENAI_API_KEY environment variable.",
            "status": "error",
        }
        print(f"[cody-graph] clippy_agent: ERROR - {result_state['last_output']}", flush=True)
        print("[cody-graph] clippy_agent: END (error)", flush=True)
        return result_state

    config = _load_config(state.get("repo_path", ""))
    current_iteration = int(state.get("phase_iteration", 0) or 0)
    max_iterations = int(config.get("max_phase_iterations", DEFAULT_MAX_PHASE_ITERATIONS))
    print(
        f"[cody-graph] [DIAG] Phase iteration: {current_iteration}/{max_iterations}",
        flush=True,
    )

    if current_iteration >= max_iterations:
        error_msg = (
            f"Phase '{phase}' exceeded max iterations ({max_iterations}). "
            "Stopping to avoid non-deterministic infinite retry loop."
        )
        result_state = {
            **state,
            "last_command": "clippy_llm_think",
            "last_output": error_msg,
            "status": "error",
        }
        print(f"[cody-graph] clippy_agent: ERROR - {error_msg}", flush=True)
        print("[cody-graph] clippy_agent: END (error)", flush=True)
        return result_state
    model = _select_model(config, phase)
    print(f"[cody-graph] [DIAG] Using model: {model} for phase '{phase}'", flush=True)
    client = OpenAI(api_key=api_key)

    system_prompt = _get_system_prompt_for_phase(phase)
    
    # We inject the last tool output (either code or clippy errors) 
    # as a "user" message so the LLM reacts to the current state.
    last_output = state.get("last_output", "") or ""
    all_issues_raw, first_file, first_line = _extract_all_issues(last_output)
    
    # Filter out already-attempted warnings
    attempted = state.get("attempted_warnings", []) or []
    current_warning_signature = None
    
    is_test_repair = state.get("last_command") == "cargo_test" and int(state.get("consecutive_test_failures", 0) or 0) > 0

    if isinstance(all_issues_raw, list):
        # New format: list of dicts
        all_issues = _filter_attempted_issues(all_issues_raw, attempted)
        
        if not all_issues and all_issues_raw and not is_test_repair:
            # All warnings have been attempted - mark phase as complete
            print(f"[cody-graph] [DIAG] All clippy warnings have been attempted. Moving to next phase.", flush=True)
            result_state = {
                **state,
                "last_command": "clippy_llm_think",
                "last_output": "All clippy warnings have been attempted.",
                "status": "ok",
                "phase_iteration": current_iteration + 1,
            }
            print("[cody-graph] clippy_agent: END (all warnings attempted)", flush=True)
            return result_state
        
        print(f"[cody-graph] [DIAG] Filtered issues: {len(all_issues)}/{len(all_issues_raw)} (attempted: {len(attempted)})", flush=True)
        
        # Debug: show all filtered issues
        for idx, issue in enumerate(all_issues):
            sig = issue.get("signature") or "(no signature)"
            print(f"[cody-graph] [DIAG] Issue {idx + 1}: file={issue.get('file')}, line={issue.get('line')}, sig={sig[:80]}", flush=True)
        
        # Format issues for LLM and store signature of current warning
        all_issues_text = "\n\n".join(f"ISSUE {idx + 1}:\n{issue['text']}" for idx, issue in enumerate(all_issues))
        file_path = all_issues[0]["file"] if all_issues else first_file
        line_no = all_issues[0]["line"] if all_issues else first_line
        current_warning_signature = all_issues[0].get("signature") if all_issues else None
        
        print(f"[cody-graph] [DIAG] Current warning signature: {current_warning_signature}", flush=True)
    else:
        # Old format or no issues
        all_issues_text = all_issues_raw
        file_path = first_file
        line_no = first_line
    
    repo_path = state.get("repo_path", "")
    snippet = _read_context_snippet(repo_path, file_path, line_no)

    context_parts = []
    if is_test_repair:
        changed_files = state.get("changed_files", []) or []
        context_parts.append(
            "TEST FAILURE REPAIR MODE:\n"
            "A recent code change caused tests to fail. Decide whether the code change is valid and tests should be updated, "
            "or whether tests found a real bug and code must be fixed. Make ONE minimal fix now."
        )
        context_parts.append("TEST OUTPUT:\n" + last_output)
        if changed_files:
            context_parts.append("CHANGED FILES:\n" + "\n".join(changed_files[:10]))
            snippet_blocks: list[str] = []
            for changed in changed_files[:2]:
                block = _read_file_head_snippet(repo_path, changed, max_lines=120)
                if block:
                    snippet_blocks.append(block)
            if snippet_blocks:
                context_parts.append("CHANGED FILE CODE CONTEXT:\n\n" + "\n\n".join(snippet_blocks))

    if all_issues_text:
        issue_count = all_issues_text.count("ISSUE ")
        context_parts.append(f"ALL_CLIPPY_ISSUES ({issue_count} total):\n" + all_issues_text)
        if issue_count > 1:
            context_parts.append(
                "\nIMPORTANT: Multiple issues present. "
                "Pick the SIMPLEST one that requires the FEWEST lines changed. "
                "Make ONE incremental fix only."
            )
    if snippet:
        context_parts.append("CODE_CONTEXT:\n" + snippet)

    current_context = {
        "role": "user",
        "content": "\n\n".join(context_parts) if context_parts else "CLIPPY_OUTPUT:\n" + last_output,
    }
    
    print(f"[cody-graph] [DIAG] Context size: {len(current_context['content'])} chars", flush=True)
    print(f"[cody-graph] [DIAG] File with warning: {file_path}, line: {line_no}", flush=True)
    
    messages = [
        {"role": "system", "content": system_prompt},
        *state["messages"],
        current_context,
    ]
    
    print(f"[cody-graph] [DIAG] Total messages in context: {len(messages)}", flush=True)
    
    try:
        print("[cody-graph] [DIAG] Calling OpenAI API...", flush=True)
        resp = client.chat.completions.create(
            model=model,
            messages=messages,
        )
        reply = resp.choices[0].message.content
        print(f"[cody-graph] [DIAG] Received response: {len(reply)} chars", flush=True)
        
        # Save LLM response for debugging
        logs_dir = state.get("logs_dir") or os.path.join(state.get("repo_path", ""), ".cody_logs")
        os.makedirs(logs_dir, exist_ok=True)
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        response_file = os.path.join(logs_dir, f"{timestamp}_llm_response.txt")
        with open(response_file, "w") as f:
            f.write("=== SYSTEM PROMPT ===\n")
            f.write(system_prompt + "\n\n")
            f.write("=== INPUT CONTEXT ===\n")
            f.write(current_context["content"] + "\n\n")
            f.write("=== LLM RESPONSE ===\n")
            f.write(reply)
        print(f"[cody-graph] [DIAG] Saved LLM response: {response_file}", flush=True)
        
    except Exception as e:
        error_msg = f"Clippy agent API error: {e}"
        result_state = {
            **state,
            "last_command": "clippy_llm_think",
            "last_output": error_msg,
            "status": "error",
            "phase_iteration": current_iteration + 1,
        }
        print(f"[cody-graph] clippy_agent: ERROR - {error_msg}", flush=True)
        print("[cody-graph] clippy_agent: END (error)", flush=True)
        return result_state

    # Append the assistant's thought process to the history
    new_messages = state["messages"] + [{"role": "assistant", "content": reply}]
    
    result_state = {
        **state,
        "messages": new_messages,
        "last_command": "clippy_llm_think",
        "llm_response": reply,
        "status": "pending",
        "phase_iteration": current_iteration + 1,
        "current_warning_signature": current_warning_signature,
    }
    print("[cody-graph] [DIAG] LLM response contains '```diff': ", "```diff" in reply, flush=True)
    print("[cody-graph] clippy_agent: END (ok)", flush=True)
    return result_state