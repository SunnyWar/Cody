import json
import os
import subprocess
from pathlib import Path
from typing import Optional, Tuple

from openai import OpenAI

from state.cody_state import CodyState

DEFAULT_MAX_PHASE_ITERATIONS = 8

REFRACTORING_STRATEGIES = [
    {
        "name": "Move functions/structs across files",
        "instruction": (
            "Move one cohesive function or struct (and any tightly coupled helpers) to another existing "
            "file or to one new file to reduce complexity and improve organization. Keep behavior identical. "
            "If no clear organizational improvement exists, do not change code and explain why."
        ),
    },
    {
        "name": "Replace with more idiomatic Rust",
        "instruction": (
            "Replace one small code section with a more idiomatic Rust equivalent while keeping behavior "
            "functionally identical. If no meaningful idiomatic improvement exists, do not change code and explain why."
        ),
    },
    {
        "name": "Rename for expressiveness",
        "instruction": (
            "Rename one function, variable, or type to make intent clearer and more expressive, updating all "
            "affected references. Keep behavior identical. If no meaningful naming improvement exists, do not "
            "change code and explain why."
        ),
    },
    {
        "name": "Reorder functions for readability",
        "instruction": (
            "Reorder functions within a single file to improve logical flow and readability without changing behavior. "
            "If no readability gain is clear, do not change code and explain why."
        ),
    },
    {
        "name": "Add intent comments",
        "instruction": (
            "Add concise, high-value comments that explain intent of non-obvious blocks/functions. Do not add noisy "
            "comments. If comments would not materially improve understanding, do not change code and explain why."
        ),
    },
]

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

def _collect_refactoring_file_context(repo_path: str, max_files: int = 3, max_lines: int = 140) -> str:
    root = Path(repo_path)
    if not root.exists():
        return ""

    candidates: list[Path] = []
    for pattern in ("engine/src/**/*.rs", "bitboard/src/**/*.rs"):
        candidates.extend(p for p in root.glob(pattern) if p.is_file())

    if not candidates:
        return ""

    # Prefer larger files first for richer refactoring opportunities.
    ranked = sorted(candidates, key=lambda p: (-p.stat().st_size, str(p)))
    snippets: list[str] = []
    for path in ranked[:max_files]:
        block = _read_file_head_snippet(repo_path, str(path), max_lines=max_lines)
        if block:
            snippets.append(block)
    return "\n\n".join(snippets)

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
- External dependencies are allowed only when they are extremely high-performance and used in performance-critical paths.
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
- You will receive one specific REFACTORING STRATEGY in the user context. Follow that strategy exactly.
- If the strategy would not produce a clear, measurable improvement, do NOT change code.
- When not changing code, respond with a short explanation only (no diff block).
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
        "UCIfeatures": """
You are Cody's UCI Protocol Agent.
Goal: Implement missing UCI commands or extend existing ones to fully support the Universal Chess Interface protocol.

PRIORITY: Focus on commands and features most commonly used in chess tournaments:
1. Core commands: position, go, stop, setoption, isready, ucinewgame
2. Time management: wtime, btime, winc, binc, movestogo, movetime
3. Search options: depth, nodes, mate, infinite
4. Engine options: Hash, Threads, MultiPV, Ponder
5. Info output: depth, seldepth, score, nodes, nps, time, pv, hashfull

CONTEXT PROVIDED:
1. Architecture: Design patterns and constraints.
2. Current UCI implementation: engine/src/api/uciapi.rs
3. UCI Protocol Specification

STRICT RULES:
- Follow the fixed-block arena allocation model.
- Ensure backward compatibility with existing UCI commands.
- Provide changes as a UNIFIED DIFF in a markdown diff block.
- REQUIRED FORMAT EXAMPLE:
    --- a/engine/src/api/uciapi.rs
    +++ b/engine/src/api/uciapi.rs
    @@ -157,1 +157,1 @@
    -    let mut moves_vec = moves;
    +    let moves_vec = moves;
- The @@ hunk header MUST include actual line numbers like: @@ -157,1 +157,1 @@
- NEVER use @@ @@ without numbers - this is INVALID
- Do NOT use *** markers
- Put your diff inside a markdown code block with 'diff' language tag
- NEVER add #[allow(...)], #[warn(...)], or any suppression attributes.
- Each UCI implementation must pass all tests and be tournament-ready.
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
    config_max_iterations = int(config.get("max_phase_iterations", DEFAULT_MAX_PHASE_ITERATIONS))
    if phase == "refactoring":
        max_iterations = len(REFRACTORING_STRATEGIES)
    else:
        max_iterations = config_max_iterations
    print(
        f"[cody-graph] [DIAG] Phase iteration: {current_iteration}/{max_iterations}",
        flush=True,
    )

    if current_iteration >= max_iterations:
        if phase == "refactoring":
            done_msg = (
                "Refactoring phase completed: all configured refactor strategies were attempted "
                "and no clearly beneficial change was found."
            )
            result_state = {
                **state,
                "last_command": "clippy_llm_think",
                "last_output": done_msg,
                "status": "ok",
            }
            print(f"[cody-graph] [DIAG] {done_msg}", flush=True)
            print("[cody-graph] clippy_agent: END (no-op complete)", flush=True)
            return result_state

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
    
    # We inject relevant context as a "user" message so the LLM reacts to current state.
    last_output = state.get("last_output", "") or ""
    attempted = state.get("attempted_warnings", []) or []
    current_warning_signature = None
    repo_path = state.get("repo_path", "")
    file_path = None
    line_no = None

    if phase == "refactoring":
        strategy = REFRACTORING_STRATEGIES[current_iteration]
        print(
            f"[cody-graph] [DIAG] Refactor strategy {current_iteration + 1}/{len(REFRACTORING_STRATEGIES)}: {strategy['name']}",
            flush=True,
        )
        refactor_context = _collect_refactoring_file_context(repo_path)
        context_parts = [
            f"REFACTORING_STRATEGY ({current_iteration + 1}/{len(REFRACTORING_STRATEGIES)} - hardest_to_easiest): {strategy['name']}",
            "TASK:\n" + strategy["instruction"],
            "IMPORTANT:\nApply exactly one small refactor. If no real benefit is found, do not create a diff.",
        ]
        if refactor_context:
            context_parts.append("RUST_CODE_CONTEXT:\n" + refactor_context)
        current_context = {
            "role": "user",
            "content": "\n\n".join(context_parts),
        }
    else:
        all_issues_raw, first_file, first_line = _extract_all_issues(last_output)
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

            # Find first issue with valid signature (not None)
            first_issue_with_sig = None
            for issue in all_issues:
                if issue.get("signature"):
                    first_issue_with_sig = issue
                    break

            if first_issue_with_sig:
                file_path = first_issue_with_sig["file"]
                line_no = first_issue_with_sig["line"]
                current_warning_signature = first_issue_with_sig.get("signature")
            else:
                file_path = all_issues[0]["file"] if all_issues else first_file
                line_no = all_issues[0]["line"] if all_issues else first_line
                current_warning_signature = None

            print(f"[cody-graph] [DIAG] Current warning signature: {current_warning_signature}", flush=True)
        else:
            # Old format or no issues
            all_issues_text = all_issues_raw
            file_path = first_file
            line_no = first_line

        snippet = _read_context_snippet(repo_path, file_path, line_no)

        context_parts = []

        # Check if we're in a build repair attempt
        repair_attempt_num = int(state.get("repair_attempts", 0) or 0)
        is_build_repair = state.get("last_command") == "cargo_build" and repair_attempt_num > 0

        if is_build_repair:
            context_parts.append(
                f"BUILD REPAIR MODE (Attempt #{repair_attempt_num}):\n"
                f"The previous code change caused the build to fail. "
                f"The build error is shown below. "
                f"Generate a minimal fix that resolves the build error while keeping the intended change. "
                f"Do NOT revert the entire change - instead, fix the compiler error."
            )

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
        with open(response_file, "w", encoding="utf-8") as f:
            f.write("=== SYSTEM PROMPT ===\n")
            f.write(system_prompt + "\n\n")
            f.write("=== INPUT CONTEXT ===\n")
            f.write(current_context["content"] + "\n\n")
            f.write("=== LLM RESPONSE ===\n")
            f.write(reply)
        print(f"[cody-graph] [DIAG] Saved LLM response: {response_file}", flush=True)
        
    except Exception as e:
        # Safely encode error message to avoid charmap issues on Windows
        try:
            error_msg = f"Clippy agent API error: {e}"
            error_msg_safe = error_msg.encode('ascii', 'replace').decode('ascii')
        except Exception:
            error_msg_safe = "Clippy agent API error: (encoding error in exception)"
        
        result_state = {
            **state,
            "last_command": "clippy_llm_think",
            "last_output": error_msg_safe,
            "status": "error",
            "phase_iteration": current_iteration + 1,
        }
        print(f"[cody-graph] clippy_agent: ERROR - {error_msg_safe}", flush=True)
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