# main.py
import os
from pathlib import Path

from graph.cody_graph import app
from state.cody_state import CodyState

repo_root = Path(os.environ.get("CODY_REPO_PATH", Path(__file__).resolve().parents[1]))

initial_state: CodyState = {
    "messages": [
        {
            "role": "user",
            "content": "Please start by reducing clippy warnings in this project.",
        }
    ],
    "repo_path": str(repo_root),
    "last_command": None,
    "last_output": None,
    "status": "pending",
}

result = app.invoke(initial_state)

status = result["status"]
last_command = result.get("last_command")
if status == "ok":
    summary = "Success: clippy, build, and tests passed."
elif last_command == "rollback":
    summary = "Rollback applied after validation failure."
else:
    summary = "Stopped with errors."

print("STATUS:", status)
print("SUMMARY:", summary)
print("LLM MESSAGES:")
for m in result["messages"]:
    print(m["role"], ":", m["content"][:200], "...\n")
print("LAST OUTPUT:")
print(result["last_output"][:1000])
