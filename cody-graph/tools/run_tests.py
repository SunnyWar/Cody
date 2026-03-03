import subprocess
import textwrap
import os
import re
from datetime import datetime
from pathlib import Path

from state.cody_state import CodyState


def run_tests(state: CodyState) -> CodyState:
    print("[cody-graph] run_tests: START", flush=True)
    repo = state["repo_path"]
    logs_dir = state.get("logs_dir") or os.path.join(repo, ".cody_logs")
    os.makedirs(logs_dir, exist_ok=True)
    state["logs_dir"] = logs_dir
    
    try:
        print("[cody-graph] [DIAG] Running 'cargo test'...", flush=True)
        result = subprocess.run(
            ["cargo", "test"],
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
        
        # Count test failures
        failed_count = len(re.findall(r"FAILED", output))
        passed_count = len(re.findall(r"test result: ok", output))
        print(f"[cody-graph] [DIAG] Tests failed: {failed_count}", flush=True)
        print(f"[cody-graph] [DIAG] Exit code: {result.returncode}", flush=True)
        
        # Save test output
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        test_log = os.path.join(logs_dir, f"{timestamp}_test_output.txt")
        with open(test_log, "w") as f:
            f.write(output)
        print(f"[cody-graph] [DIAG] Saved test output to: {test_log}", flush=True)
        
        status = "ok" if result.returncode == 0 else "error"
    except Exception as e:
        output = f"Exception while running cargo test: {e}"
        status = "error"
        print(f"[cody-graph] [DIAG] Exception: {e}", flush=True)

    result_state = {
        **state,
        "last_output": output,
        "last_command": "cargo_test",
        "status": status,
    }
    if status != "ok":
        print(f"[cody-graph] run_tests: ERROR (exit code {result.returncode})", flush=True)
    print(f"[cody-graph] run_tests: END ({status})", flush=True)
    return result_state
