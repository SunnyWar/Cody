import subprocess
import textwrap
import os
import re
from datetime import datetime
from pathlib import Path

from state.cody_state import CodyState
from retry_manager import create_retry_manager


def run_build(state: CodyState) -> CodyState:
    print("[cody-graph] run_build: START", flush=True)
    repo = state["repo_path"]
    logs_dir = state.get("logs_dir") or os.path.join(repo, ".cody_logs")
    os.makedirs(logs_dir, exist_ok=True)
    state["logs_dir"] = logs_dir
    
    try:
        print("[cody-graph] [DIAG] Running 'cargo build'...", flush=True)
        result = subprocess.run(
            ["cargo", "build"],
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
        
        # Count build errors
        error_count = len(re.findall(r"^error:", output, re.MULTILINE))
        print(f"[cody-graph] [DIAG] Build errors: {error_count}", flush=True)
        print(f"[cody-graph] [DIAG] Exit code: {result.returncode}", flush=True)
        
        # Save build output
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        build_log = os.path.join(logs_dir, f"{timestamp}_build_output.txt")
        with open(build_log, "w") as f:
            f.write(output)
        print(f"[cody-graph] [DIAG] Saved build output to: {build_log}", flush=True)
        
        status = "ok" if result.returncode == 0 else "error"
    except Exception as e:
        output = f"Exception while running cargo build: {e}"
        status = "error"
        print(f"[cody-graph] [DIAG] Exception: {e}", flush=True)

    result_state = {
        **state,
        "last_output": output,
        "last_command": "cargo_build",
        "status": status,
    }
    
    # Security gate: If build failed after a patch, mark for repair routing
    if status != "ok" and state.get("last_diff"):
        retry_mgr = create_retry_manager(state)
        if retry_mgr.should_attempt_repair(result_state):
            print("[cody-graph] [DIAG] Build failed after patch - marking for LLM repair attempt", flush=True)
            result_state["build_failed_needs_repair"] = True
        else:
            print("[cody-graph] [DIAG] Build failed - repair attempts exhausted, will rollback", flush=True)
    
    if status != "ok":
        print(f"[cody-graph] run_build: ERROR (exit code {result.returncode})", flush=True)
    print(f"[cody-graph] run_build: END ({status})", flush=True)
    return result_state
