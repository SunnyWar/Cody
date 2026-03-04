import os
import re
import shutil
import subprocess
from datetime import datetime
from pathlib import Path

def _sanitize_diff(diff_content: str) -> str:
    """Clean up diff format if LLM generated non-standard formatting.
    
    Handles:
    - *** Begin/End Patch markers
    - Missing --- +++ headers
    - Improper @@ markers
    """
    lines = diff_content.strip().split('\n')
    
    # Remove *** Begin Patch and *** End Patch markers
    lines = [l for l in lines if not l.startswith('*** Begin') and not l.startswith('*** End')]
    
    cleaned = []
    for line in lines:
        # Convert *** markers to --- +++ format
        if line.startswith('*** Update File:'):
            filename = line.replace('*** Update File:', '').strip()
            cleaned.append(f'--- a/{filename}')
            cleaned.append(f'+++ b/{filename}')
        else:
            cleaned.append(line)
    
    return '\n'.join(cleaned)

def _ensure_logs_dir(repo_path: str) -> str:
    """Ensure .cody_logs directory exists and return its path."""
    logs_dir = os.path.join(repo_path, ".cody_logs")
    os.makedirs(logs_dir, exist_ok=True)
    return logs_dir

def _save_diagnostic(logs_dir: str, name: str, content: str) -> None:
    """Save a diagnostic file with timestamp."""
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    filename = f"{timestamp}_{name}.log"
    filepath = os.path.join(logs_dir, filename)
    with open(filepath, "w") as f:
        f.write(content)
    print(f"[cody-graph] [DIAG] Saved: {filename}", flush=True)

def apply_diff(state: dict) -> dict:
    """
    Parses the last assistant message for a unified diff and applies it.
    Enhanced with detailed diagnostics and logging.
    """
    print("[cody-graph] apply_diff: START", flush=True)
    repo_path = state["repo_path"]
    logs_dir = _ensure_logs_dir(repo_path)
    state["logs_dir"] = logs_dir
    
    messages = state.get("messages", [])
    if not messages:
        error_msg = "No messages found."
        result = {**state, "status": "error", "last_output": error_msg}
        print(f"[cody-graph] apply_diff: ERROR - {error_msg}", flush=True)
        print("[cody-graph] apply_diff: END (error)", flush=True)
        return result

    last_reply = messages[-1]["content"]
    print(f"[cody-graph] [DIAG] LLM response length: {len(last_reply)} chars", flush=True)
    _save_diagnostic(logs_dir, "llm_response_raw", last_reply)

    # Extract the diff block (looking for ```diff ... ``` or raw diff headers)
    print("[cody-graph] [DIAG] Attempting to extract diff from markdown block...", flush=True)
    diff_match = re.search(r"```(?:diff)?\n(.*?)\n```", last_reply, re.DOTALL)
    diff_content = diff_match.group(1) if diff_match else None

    if not diff_content:
        print("[cody-graph] [DIAG] No markdown diff block found, checking for raw diff headers...", flush=True)
        # If no markdown block, try to find raw diff headers
        if "---" in last_reply and "+++" in last_reply:
            diff_content = last_reply
            print("[cody-graph] [DIAG] Found raw diff headers", flush=True)
        else:
            error_msg = "No unified diff found in LLM response."
            print(f"[cody-graph] [DIAG] ERROR - {error_msg}", flush=True)
            _save_diagnostic(logs_dir, "diff_extraction_failed", last_reply)
            result = {**state, "status": "pending", "last_output": error_msg}
            print("[cody-graph] apply_diff: END (no diff)", flush=True)
            return result

    print(f"[cody-graph] [DIAG] Extracted diff content: {len(diff_content)} chars", flush=True)
    
    # Sanitize diff format (handle LLM-generated non-standard formats)
    print("[cody-graph] [DIAG] Sanitizing diff format...", flush=True)
    diff_content = _sanitize_diff(diff_content)
    print(f"[cody-graph] [DIAG] After sanitization: {len(diff_content)} chars", flush=True)
    
    _save_diagnostic(logs_dir, "diff_extracted", diff_content)

    try:
        # Write the diff to a temporary file
        patch_path = os.path.join(repo_path, "changes.patch")
        print(f"[cody-graph] [DIAG] Writing patch to: {patch_path}", flush=True)
        with open(patch_path, "w") as f:
            f.write(diff_content)

        print("[cody-graph] [DIAG] Looking for patch tool (git or patch)...", flush=True)
        if shutil.which("git"):
            print("[cody-graph] [DIAG] Using 'git apply'", flush=True)
            result = subprocess.run(
                ["git", "apply", "--whitespace=nowarn", "changes.patch"],
                cwd=repo_path,
                capture_output=True,
                text=True,
            )
        elif shutil.which("patch"):
            print("[cody-graph] [DIAG] Using 'patch' command", flush=True)
            result = subprocess.run(
                ["patch", "-p1", "-i", "changes.patch"],
                cwd=repo_path,
                capture_output=True,
                text=True,
            )
        else:
            os.remove(patch_path)
            error_msg = "No patching tool found (git or patch)."
            print(f"[cody-graph] apply_diff: ERROR - {error_msg}", flush=True)
            result_state = {
                **state,
                "status": "error",
                "last_output": error_msg,
            }
            print("[cody-graph] apply_diff: END (error)", flush=True)
            return result_state

        print(f"[cody-graph] [DIAG] Patch tool exit code: {result.returncode}", flush=True)
        if result.stdout:
            print(f"[cody-graph] [DIAG] Patch stdout: {result.stdout[:500]}", flush=True)
            _save_diagnostic(logs_dir, "patch_stdout", result.stdout)
        if result.stderr:
            print(f"[cody-graph] [DIAG] Patch stderr: {result.stderr[:500]}", flush=True)
            _save_diagnostic(logs_dir, "patch_stderr", result.stderr)

        os.remove(patch_path) # Clean up

        if result.returncode == 0:
            success_msg = "Patch applied successfully."
            print(f"[cody-graph] apply_diff: SUCCESS - {success_msg}", flush=True)
            result_state = {
                **state,
                "status": "pending",
                "last_output": success_msg,
                "last_diff": diff_content,
                "last_command": "apply_diff",
            }
            print("[cody-graph] apply_diff: END (ok)", flush=True)
            return result_state
        else:
            error_details = f"Patch failed with exit code {result.returncode}: {result.stderr}"
            print(f"[cody-graph] apply_diff: ERROR - {error_details[:200]}", flush=True)
            result_state = {
                **state,
                "status": "error",
                "last_output": error_details,
            }
            print("[cody-graph] apply_diff: END (error)", flush=True)
            return result_state

    except Exception as e:
        error_msg = f"Error applying patch: {e}"
        print(f"[cody-graph] apply_diff: ERROR - {error_msg}", flush=True)
        _save_diagnostic(logs_dir, "patch_exception", str(e))
        result_state = {**state, "status": "error", "last_output": error_msg}
        print("[cody-graph] apply_diff: END (error)", flush=True)
        return result_state