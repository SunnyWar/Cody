import subprocess
import textwrap
import os
from datetime import datetime
from pathlib import Path

from state.cody_state import CodyState


def run_fmt(state: CodyState) -> CodyState:
    print("[cody-graph] run_fmt: START", flush=True)
    repo = state["repo_path"]
    logs_dir = state.get("logs_dir") or os.path.join(repo, ".cody_logs")
    os.makedirs(logs_dir, exist_ok=True)
    state["logs_dir"] = logs_dir
    
    try:
        print("[cody-graph] [DIAG] Running 'cargo fmt'...", flush=True)
        result = subprocess.run(
            ["cargo", "fmt"],
            cwd=repo,
            capture_output=True,
            text=True,
            check=False,
        )
        output = textwrap.dedent(f"""
        exit_code: {result.returncode}
        stdout:
        {result.stdout}

        stderr:
        {result.stderr}
        """)
        
        print(f"[cody-graph] [DIAG] Formatting exit code: {result.returncode}", flush=True)
        
        # Save fmt output
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        fmt_log = os.path.join(logs_dir, f"{timestamp}_fmt_output.txt")
        with open(fmt_log, "w") as f:
            f.write(output)
        print(f"[cody-graph] [DIAG] Saved fmt output to: {fmt_log}", flush=True)
        
        status = "ok" if result.returncode == 0 else "error"
    except Exception as e:
        output = f"Exception while running cargo fmt: {e}"
        status = "error"
        print(f"[cody-graph] [DIAG] Exception: {e}", flush=True)

    result_state = {
        **state,
        "last_output": output,
        "last_command": "cargo_fmt",
        "status": status,
    }
    if status != "ok":
        print(f"[cody-graph] run_fmt: ERROR (exit code {result.returncode})", flush=True)
    else:
        print(f"[cody-graph] run_fmt: SUCCESS", flush=True)
    print(f"[cody-graph] run_fmt: END ({status})", flush=True)
    return result_state
