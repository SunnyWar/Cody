# main.py
from graph.cody_graph import app
from state.cody_state import CodyState

initial_state: CodyState = {
    "messages": [
        {
            "role": "user",
            "content": "Please start by reducing clippy warnings in this project.",
        }
    ],
    "repo_path": "/path/to/cody",
    "last_command": None,
    "last_output": None,
    "status": "pending",
}

result = app.invoke(initial_state)

print("STATUS:", result["status"])
print("LLM MESSAGES:")
for m in result["messages"]:
    print(m["role"], ":", m["content"][:200], "...\n")
print("CLIPPY OUTPUT:")
print(result["last_output"][:1000])
