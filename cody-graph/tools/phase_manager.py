import json
import os
from pathlib import Path
from datetime import datetime

def save_phase_state(repo_path: str, state: dict) -> None:
    """Save current phase state to orchestrator_state.json for resumption."""
    orchestrator_state = {
        "current_phase": state.get("current_phase", "clippy"),
        "phases_completed": state.get("phases_completed", []),
        "phases_todo": state.get("phases_todo", []),
        "phase_iteration": state.get("phase_iteration", 0),
        "status": state.get("status", "pending"),
        "last_update": datetime.now().isoformat(),
        "run_count": 0,  # Placeholder for future tracking
    }
    
    state_file = os.path.join(repo_path, "orchestrator_state.json")
    try:
        with open(state_file, "w") as f:
            json.dump(orchestrator_state, f, indent=2)
        print(f"[cody-graph] [DIAG] Saved phase state to {state_file}", flush=True)
    except Exception as e:
        print(f"[cody-graph] [DIAG] Error saving phase state: {e}", flush=True)

def load_phase_state(repo_path: str) -> dict:
    """Load phase state from orchestrator_state.json if it exists (for resuming)."""
    state_file = os.path.join(repo_path, "orchestrator_state.json")
    if os.path.exists(state_file):
        try:
            with open(state_file, "r") as f:
                return json.load(f)
        except Exception as e:
            print(f"[cody-graph] [DIAG] Error loading phase state: {e}", flush=True)
    return {}
