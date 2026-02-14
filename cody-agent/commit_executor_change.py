"""
Commit the most recent executor change.

Stages only the files recorded by the executor and generates a commit message.
"""

import argparse
import subprocess
import sys
from pathlib import Path
from typing import List, Optional

from todo_manager import TodoList
from executor_state import read_last_change


PHASE_LABELS = {
    "refactoring": "Refactor",
    "performance": "Perf",
    "features": "Feature",
    "clippy": "Clippy",
}


def _git_has_changes(repo_root: Path, file_path: str) -> bool:
    """Return True if the file has staged or unstaged changes."""
    for args in (["git", "diff", "--name-only", "--", file_path],
                 ["git", "diff", "--name-only", "--cached", "--", file_path]):
        result = subprocess.run(
            args,
            cwd=repo_root,
            capture_output=True,
            text=True,
            check=False,
        )
        if result.stdout.strip():
            return True
    return False


def _stage_files(repo_root: Path, files: List[str]) -> bool:
    """Stage only the specified files."""
    staged_any = False
    for file_path in files:
        if not _git_has_changes(repo_root, file_path):
            continue
        result = subprocess.run(
            ["git", "add", "--", file_path],
            cwd=repo_root,
            capture_output=True,
            text=True,
            check=False,
        )
        if result.returncode != 0:
            print(f"ERROR: Failed to stage {file_path}: {result.stderr.strip()}")
            return False
        staged_any = True
    return staged_any


def _build_commit_message(repo_root: Path, phase: str, item_id: str, fallback: Optional[str]) -> str:
    """Create a commit message from TODO metadata."""
    if fallback:
        return fallback.strip()

    label = PHASE_LABELS.get(phase, phase.title())
    title = ""

    todo_list = TodoList(phase, repo_root)
    for item in todo_list.items:
        if item.id == item_id:
            title = item.title
            break

    if title:
        message = f"{label}: {item_id} {title}"
    else:
        message = f"{label}: {item_id}"

    max_len = 72
    if len(message) > max_len:
        message = message[: max_len - 3].rstrip() + "..."

    return message


def _parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Commit the last executor change.")
    parser.add_argument("--fallback-message", dest="fallback", default=None)
    return parser.parse_args()


def main() -> int:
    args = _parse_args()
    repo_root = Path(__file__).parent.parent

    state = read_last_change(repo_root)
    if not state:
        print("ERROR: No executor change recorded. Run an executor first.")
        return 1

    phase = state["phase"]
    item_id = state["item_id"]
    files = state["files"]

    if not _stage_files(repo_root, files):
        print("ERROR: No changes staged for the recorded files.")
        return 1

    message = _build_commit_message(repo_root, phase, item_id, args.fallback)

    result = subprocess.run(
        ["git", "commit", "-m", message],
        cwd=repo_root,
        capture_output=True,
        text=True,
        check=False,
    )

    if result.returncode != 0:
        print("ERROR: Git commit failed.")
        print(result.stderr.strip())
        return 1

    print(f"Committed: {message}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
