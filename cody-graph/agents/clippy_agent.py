import json
import os
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

def _select_model(config: dict) -> str:
    models = config.get("models", {}) if isinstance(config, dict) else {}
    return models.get("clippy") or config.get("model") or "gpt-4o-mini"

def _extract_first_warning(output: str) -> Tuple[str, Optional[str], Optional[int]]:
    lines = output.splitlines()
    start_idx = None
    for i, line in enumerate(lines):
        stripped = line.lstrip()
        if stripped.startswith("warning:") or stripped.startswith("error:"):
            start_idx = i
            break

    if start_idx is None:
        return (output.strip(), None, None)

    block: list[str] = []
    file_path: Optional[str] = None
    line_no: Optional[int] = None

    for j in range(start_idx, len(lines)):
        line = lines[j]
        stripped = line.lstrip()
        if j > start_idx and (stripped.startswith("warning:") or stripped.startswith("error:")):
            break
        block.append(line)
        if "-->" in line:
            arrow = line.split("-->", 1)[1].strip()
            parts = arrow.rsplit(":", 2)
            if len(parts) == 3:
                file_path = parts[0].strip()
                try:
                    line_no = int(parts[1])
                except ValueError:
                    line_no = None

    return ("\n".join(block).strip(), file_path, line_no)

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
- Fix one warning/error at a time.
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
- Each feature should pass all tests.
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
    model = _select_model(config)
    print(f"[cody-graph] [DIAG] Using model: {model} for phase '{phase}'", flush=True)
    client = OpenAI(api_key=api_key)

    system_prompt = _get_system_prompt_for_phase(phase)
    
    # We inject the last tool output (either code or clippy errors) 
    # as a "user" message so the LLM reacts to the current state.
    last_output = state.get("last_output", "") or ""
    warning_text, file_path, line_no = _extract_first_warning(last_output)
    snippet = _read_context_snippet(state.get("repo_path", ""), file_path, line_no)

    context_parts = []
    if warning_text:
        context_parts.append("FIRST_CLIPPY_WARNING:\n" + warning_text)
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
    }
    print("[cody-graph] [DIAG] LLM response contains '```diff': ", "```diff" in reply, flush=True)
    print("[cody-graph] clippy_agent: END (ok)", flush=True)
    return result_state