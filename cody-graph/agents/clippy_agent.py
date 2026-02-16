import json
import os
from pathlib import Path
from typing import Optional, Tuple

from openai import OpenAI

from state.cody_state import CodyState

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
        if line.lstrip().startswith("warning:"):
            start_idx = i
            break

    if start_idx is None:
        return (output.strip(), None, None)

    block: list[str] = []
    file_path: Optional[str] = None
    line_no: Optional[int] = None

    for j in range(start_idx, len(lines)):
        line = lines[j]
        if j > start_idx and line.lstrip().startswith("warning:"):
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

def clippy_agent(state: CodyState) -> CodyState:
    print("[cody-graph] clippy_agent: start", flush=True)
    api_key = os.environ.get("OPENAI_API_KEY")
    if not api_key:
        result_state = {
            **state,
            "last_command": "clippy_llm_think",
            "last_output": "Missing OPENAI_API_KEY environment variable.",
            "status": "error",
        }
        print(f"[cody-graph] clippy_agent: error: {result_state['last_output']}", flush=True)
        print("[cody-graph] clippy_agent: end (error)", flush=True)
        return result_state

    config = _load_config(state.get("repo_path", ""))
    model = _select_model(config)
    client = OpenAI(api_key=api_key)

    system_prompt = """
You are Cody's ClippyAgent. 
Goal: Reduce Clippy warnings in this Rust project.

CONTEXT PROVIDED:
1. Source Code: You will see the content of .rs files.
2. Clippy Output: You will see the current warnings/errors.

STRICT RULES:
- Only suggest changes to existing files in the repo.
- Respond with a short explanation of the fix.
- Provide the fix in a UNIFIED DIFF format inside a ```diff code block.
- Do not suggest external dependencies.
- Fix only the single Clippy warning provided.
"""
    
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
    
    messages = [
        {"role": "system", "content": system_prompt},
        *state["messages"],
        current_context,
    ]
    
    try:
        resp = client.chat.completions.create(
            model=model,
            messages=messages,
        )
        reply = resp.choices[0].message.content
    except Exception as e:
        result_state = {
            **state,
            "last_command": "clippy_llm_think",
            "last_output": f"Clippy agent API error: {e}",
            "status": "error",
        }
        print(f"[cody-graph] clippy_agent: error: {result_state['last_output']}", flush=True)
        print("[cody-graph] clippy_agent: end (error)", flush=True)
        return result_state

    # Append the assistant's thought process to the history
    new_messages = state["messages"] + [{"role": "assistant", "content": reply}]
    
    result_state = {
        **state,
        "messages": new_messages,
        "last_command": "clippy_llm_think",
        "status": "pending",
    }
    print("[cody-graph] clippy_agent: end (ok)", flush=True)
    return result_state