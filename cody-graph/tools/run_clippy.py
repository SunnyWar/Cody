import subprocess
import textwrap
import os
import re
from datetime import datetime
from pathlib import Path

from state.cody_state import CodyState

def run_clippy(state: CodyState) -> CodyState:
    print("[cody-graph] run_clippy: START", flush=True)
    repo = state["repo_path"]
    logs_dir = state.get("logs_dir") or os.path.join(repo, ".cody_logs")
    os.makedirs(logs_dir, exist_ok=True)
    state["logs_dir"] = logs_dir
    
    try:
        print("[cody-graph] [DIAG] Running 'cargo clippy --'...", flush=True)
        result = subprocess.run(
            ["cargo", "clippy", "--", "-D", "warnings"],
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
        
        # Count warnings
        warning_count = len(re.findall(r"^warning:", output, re.MULTILINE))
        error_count = len(re.findall(r"^error:", output, re.MULTILINE))
        print(f"[cody-graph] [DIAG] Warnings: {warning_count}, Errors: {error_count}", flush=True)
        print(f"[cody-graph] [DIAG] Exit code: {result.returncode}", flush=True)
        
        # Save clippy output
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        clippy_log = os.path.join(logs_dir, f"{timestamp}_clippy_output.txt")
        with open(clippy_log, "w") as f:
            f.write(output)
        print(f"[cody-graph] [DIAG] Saved clippy output to: {clippy_log}", flush=True)
        
        status = "ok" if result.returncode == 0 else "error"
    except Exception as e:
        output = f"Exception while running clippy: {e}"
        status = "error"
        print(f"[cody-graph] [DIAG] Exception: {e}", flush=True)

    prev_best = state.get("best_clippy_error_count")
    if isinstance(prev_best, int):
        best_error_count = min(prev_best, error_count)
    else:
        best_error_count = error_count

    result_state = {
        **state,
        "last_output": output,
        "last_command": "clippy",
        "status": status,
        "clippy_error_count": error_count,
        "best_clippy_error_count": best_error_count,
    }
    if status != "ok":
        print(f"[cody-graph] run_clippy: ERROR (exit code {result.returncode})", flush=True)
        # Show first actionable issue (error preferred, then warning)
        first_error_match = re.search(r"error:.*?(?=error:|warning:|$)", output, re.DOTALL)
        if first_error_match:
            issue_preview = first_error_match.group(0)[:400]
            print(f"[cody-graph] [DIAG] First error:\n{issue_preview}...", flush=True)
        else:
            first_warning_match = re.search(r"warning:.*?(?=warning:|error:|$)", output, re.DOTALL)
            if first_warning_match:
                issue_preview = first_warning_match.group(0)[:400]
                print(f"[cody-graph] [DIAG] First warning:\n{issue_preview}...", flush=True)
    print(f"[cody-graph] run_clippy: END ({status})", flush=True)
    return result_state
