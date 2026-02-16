import os
import re
import shutil
import subprocess

def apply_diff(state: dict) -> dict:
    """
    Parses the last assistant message for a unified diff and applies it.
    """
    print("[cody-graph] apply_diff: start", flush=True)
    messages = state.get("messages", [])
    if not messages:
        result = {**state, "status": "error", "last_output": "No messages found."}
        print("[cody-graph] apply_diff: end (error)", flush=True)
        return result

    last_reply = messages[-1]["content"]
    repo_path = state["repo_path"]

    # Extract the diff block (looking for ```diff ... ``` or raw diff headers)
    diff_match = re.search(r"```(?:diff)?\n(.*?)\n```", last_reply, re.DOTALL)
    diff_content = diff_match.group(1) if diff_match else None

    if not diff_content:
        # If no markdown block, try to find raw diff headers
        if "---" in last_reply and "+++" in last_reply:
            diff_content = last_reply
        else:
            result = {**state, "status": "pending", "last_output": "No diff found to apply."}
            print("[cody-graph] apply_diff: end (no diff)", flush=True)
            return result

    try:
        # Write the diff to a temporary file
        patch_path = os.path.join(repo_path, "changes.patch")
        with open(patch_path, "w") as f:
            f.write(diff_content)

        if shutil.which("git"):
            result = subprocess.run(
                ["git", "apply", "--whitespace=nowarn", "changes.patch"],
                cwd=repo_path,
                capture_output=True,
                text=True,
            )
        elif shutil.which("patch"):
            result = subprocess.run(
                ["patch", "-p1", "-i", "changes.patch"],
                cwd=repo_path,
                capture_output=True,
                text=True,
            )
        else:
            os.remove(patch_path)
            result_state = {
                **state,
                "status": "error",
                "last_output": "No patching tool found (git or patch).",
            }
            print("[cody-graph] apply_diff: end (error)", flush=True)
            return result_state

        os.remove(patch_path) # Clean up

        if result.returncode == 0:
            result_state = {
                **state,
                "status": "pending",
                "last_output": "Patch applied successfully.",
                "last_diff": diff_content,
                "last_command": "apply_diff",
            }
            print("[cody-graph] apply_diff: end (ok)", flush=True)
            return result_state
        else:
            result_state = {**state, "status": "error", "last_output": f"Patch failed: {result.stderr}"}
            print("[cody-graph] apply_diff: end (error)", flush=True)
            return result_state

    except Exception as e:
        result_state = {**state, "status": "error", "last_output": f"Error applying patch: {e}"}
        print("[cody-graph] apply_diff: end (error)", flush=True)
        return result_state