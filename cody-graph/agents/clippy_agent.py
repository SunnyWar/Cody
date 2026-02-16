import json
import os
from pathlib import Path

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
"""
    
    # We inject the last tool output (either code or clippy errors) 
    # as a "user" message so the LLM reacts to the current state.
    current_context = {
        "role": "user", 
        "content": f"CURRENT REPO STATE/OUTPUT:\n{state['last_output']}"
    }
    
    messages = [
        {"role": "system", "content": system_prompt},
        *state["messages"],
        current_context,
    ]
    
    resp = client.chat.completions.create(
        model=model,
        messages=messages,
    )
    reply = resp.choices[0].message.content

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