"""
Executor state tracking for commit finalization.
"""

import json
from datetime import datetime
from pathlib import Path
from typing import List, Optional, Dict, Any

STATE_FILENAME = ".last_executor_change.json"


def _state_path(repo_root: Path) -> Path:
    return Path(repo_root) / "cody-agent" / STATE_FILENAME


def record_last_change(repo_root: Path, phase: str, item_id: str, files: List[str]) -> bool:
    """Record the last executor change for commit finalization."""
    if not files:
        return False

    normalized_files = []
    for file_path in files:
        if not file_path:
            continue
        normalized_files.append(str(file_path).replace("\\", "/"))

    if not normalized_files:
        return False

    data = {
        "phase": phase,
        "item_id": item_id,
        "files": normalized_files,
        "timestamp": datetime.now().isoformat(),
    }

    path = _state_path(repo_root)
    path.write_text(json.dumps(data, indent=2), encoding="utf-8")
    return True


def read_last_change(repo_root: Path) -> Optional[Dict[str, Any]]:
    """Read the last executor change for commit finalization."""
    path = _state_path(repo_root)
    if not path.exists():
        return None

    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None

    if not isinstance(data, dict):
        return None

    if not data.get("phase") or not data.get("item_id"):
        return None

    files = data.get("files")
    if not isinstance(files, list) or not files:
        return None

    return data
