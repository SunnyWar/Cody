import os
import shutil
import subprocess

from state.cody_state import CodyState


def rollback_changes(state: CodyState) -> CodyState:
    repo = state["repo_path"]
    diff_content = state.get("last_diff")
    if not diff_content:
        return {
            **state,
            "last_output": "No diff available to rollback.",
            "last_command": "rollback",
            "status": "error",
        }

    patch_path = os.path.join(repo, "rollback.patch")
    try:
        with open(patch_path, "w") as f:
            f.write(diff_content)

        if shutil.which("git"):
            result = subprocess.run(
                ["git", "apply", "-R", "--whitespace=nowarn", "rollback.patch"],
                cwd=repo,
                capture_output=True,
                text=True,
            )
        elif shutil.which("patch"):
            result = subprocess.run(
                ["patch", "-R", "-p1", "-i", "rollback.patch"],
                cwd=repo,
                capture_output=True,
                text=True,
            )
        else:
            return {
                **state,
                "last_output": "No patching tool found (git or patch).",
                "last_command": "rollback",
                "status": "error",
            }

        if result.returncode == 0:
            return {
                **state,
                "last_output": "Rollback applied.",
                "last_command": "rollback",
                "status": "error",
            }
        return {
            **state,
            "last_output": f"Rollback failed: {result.stderr}",
            "last_command": "rollback",
            "status": "error",
        }
    finally:
        if os.path.exists(patch_path):
            os.remove(patch_path)
