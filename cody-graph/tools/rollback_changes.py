import os
import shutil
import subprocess
from datetime import datetime
from pathlib import Path

from state.cody_state import CodyState


def rollback_changes(state: CodyState) -> CodyState:
    print("[cody-graph] rollback_changes: START", flush=True)
    repo = state["repo_path"]
    logs_dir = state.get("logs_dir") or os.path.join(repo, ".cody_logs")
    os.makedirs(logs_dir, exist_ok=True)
    state["logs_dir"] = logs_dir
    
    diff_content = state.get("last_diff")
    if not diff_content:
        result_state = {
            **state,
            "last_output": "No diff available to rollback.",
            "last_command": "rollback",
            "status": "error",
        }
        print(f"[cody-graph] rollback_changes: ERROR - {result_state['last_output']}", flush=True)
        print("[cody-graph] rollback_changes: END (error)", flush=True)
        return result_state

    print("[cody-graph] [DIAG] Attempting to rollback previous changes...", flush=True)
    patch_path = os.path.join(repo, "rollback.patch")
    try:
        with open(patch_path, "w") as f:
            f.write(diff_content)
        print("[cody-graph] [DIAG] Wrote patch file for rollback", flush=True)

        if shutil.which("git"):
            print("[cody-graph] [DIAG] Using 'git apply -R'", flush=True)
            result = subprocess.run(
                ["git", "apply", "-R", "--whitespace=nowarn", "rollback.patch"],
                cwd=repo,
                capture_output=True,
                text=True,
            )
        elif shutil.which("patch"):
            print("[cody-graph] [DIAG] Using 'patch -R'", flush=True)
            result = subprocess.run(
                ["patch", "-R", "-p1", "-i", "rollback.patch"],
                cwd=repo,
                capture_output=True,
                text=True,
            )
        else:
            print("[cody-graph] [DIAG] ERROR: No patching tool found", flush=True)
            return {
                **state,
                "last_output": "No patching tool found (git or patch).",
                "last_command": "rollback",
                "status": "error",
            }

        print(f"[cody-graph] [DIAG] Rollback exit code: {result.returncode}", flush=True)
        
        # Save rollback output
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        rollback_log = os.path.join(logs_dir, f"{timestamp}_rollback_output.txt")
        with open(rollback_log, "w") as f:
            f.write(f"Exit code: {result.returncode}\n")
            f.write(f"Stdout: {result.stdout}\n")
            f.write(f"Stderr: {result.stderr}\n")
        print(f"[cody-graph] [DIAG] Saved rollback output to: {rollback_log}", flush=True)
        
        if result.returncode == 0:
            result_state = {
                **state,
                "last_output": "Rollback applied successfully.",
                "last_command": "rollback",
                "status": "error",
            }
            print(f"[cody-graph] rollback_changes: SUCCESS - {result_state['last_output']}", flush=True)
            print("[cody-graph] rollback_changes: END (error)", flush=True)
            return result_state
        result_state = {
            **state,
            "last_output": f"Rollback failed: {result.stderr}",
            "last_command": "rollback",
            "status": "error",
        }
        print(f"[cody-graph] rollback_changes: ERROR - {result_state['last_output']}", flush=True)
        print("[cody-graph] rollback_changes: END (error)", flush=True)
        return result_state
    finally:
        if os.path.exists(patch_path):
            os.remove(patch_path)
