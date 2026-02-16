from openai import OpenAI
from state.cody_state import CodyState

client = OpenAI()

def clippy_agent(state: CodyState) -> CodyState:
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
        model="gpt-4o-mini",  # Corrected model name
        messages=messages,
    )
    reply = resp.choices[0].message.content

    # Append the assistant's thought process to the history
    new_messages = state["messages"] + [{"role": "assistant", "content": reply}]
    
    return {
        **state,
        "messages": new_messages,
        "last_command": "clippy_llm_think",
        "status": "pending",
    }