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
    original_command = state.get("last_command")  # Preserve context for after_rollback
    if not diff_content:
        result_state = {
            **state,
            "last_output": "No diff available to rollback.",
            "last_command": original_command,  # Preserve original context
            "status": "error",
        }
        print(f"[cody-graph] rollback_changes: ERROR - {result_state['last_output']}", flush=True)
        print("[cody-graph] rollback_changes: END (error)", flush=True)
        return result_state

    print("[cody-graph] [DIAG] Attempting to rollback previous changes...", flush=True)
    patch_path = os.path.join(repo, "rollback.patch")
    
    try:
        # Try to extract which files were modified from the patch
        modified_files = []
        if diff_content:
            for line in diff_content.splitlines():
                if line.startswith("--- a/") or line.startswith("+++ b/"):
                    # Extract file path
                    file_part = line[6:]  # Skip "--- a/" or "+++ b/"
                    if file_part and file_part not in modified_files:
                        modified_files.append(file_part)
        
        print(f"[cody-graph] [DIAG] Files to restore: {modified_files}", flush=True)
        
        # Use git checkout to restore files to HEAD state (more reliable than reverse patch)
        if shutil.which("git") and modified_files:
            print("[cody-graph] [DIAG] Using 'git checkout HEAD' to restore files", flush=True)
            # Checkout each file individually to HEAD
            result = subprocess.run(
                ["git", "checkout", "HEAD"] + modified_files,
                cwd=repo,
                capture_output=True,
                text=True,
            )
        elif shutil.which("git"):
            # Fallback: restore all modified files
            print("[cody-graph] [DIAG] Using 'git checkout HEAD' for all modified files", flush=True)
            result = subprocess.run(
                ["git", "checkout", "HEAD"],
                cwd=repo,
                capture_output=True,
                text=True,
            )
        else:
            # Last resort: try reverse patch
            print("[cody-graph] [DIAG] git not available, trying patch -R", flush=True)
            with open(patch_path, "w", encoding="utf-8") as f:
                f.write(diff_content)
            if shutil.which("patch"):
                result = subprocess.run(
                    ["patch", "-R", "-p1", "-i", "rollback.patch"],
                    cwd=repo,
                    capture_output=True,
                    text=True,
                )
            else:
                result = subprocess.CompletedProcess(
                    args=[], returncode=1, stdout="", stderr="No tools available for rollback"
                )
        
        print(f"[cody-graph] [DIAG] Rollback exit code: {result.returncode}", flush=True)
        
        # Save rollback output
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        rollback_log = os.path.join(logs_dir, f"{timestamp}_rollback_output.txt")
        with open(rollback_log, "w", encoding="utf-8") as f:
            f.write(f"Exit code: {result.returncode}\n")
            f.write(f"Stdout: {result.stdout}\n")
            f.write(f"Stderr: {result.stderr}\n")
        print(f"[cody-graph] [DIAG] Saved rollback output to: {rollback_log}", flush=True)
        
        if result.returncode == 0:
            result_state = {
                **state,
                "last_output": "Rollback applied successfully.",
                "last_command": original_command,  # Preserve original context
                "status": "error",
            }
            print(f"[cody-graph] rollback_changes: SUCCESS - {result_state['last_output']}", flush=True)
            print("[cody-graph] rollback_changes: END (error)", flush=True)
            return result_state
        result_state = {
            **state,
            "last_output": f"Rollback failed: {result.stderr}",
            "last_command": original_command,  # Preserve original context
            "status": "error",
        }
        print(f"[cody-graph] rollback_changes: ERROR - {result_state['last_output']}", flush=True)
        print("[cody-graph] rollback_changes: END (error)", flush=True)
        return result_state
    finally:
        if os.path.exists(patch_path):
            os.remove(patch_path)
