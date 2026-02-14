"""
Utilities for parsing and applying LLM code outputs.
"""

from __future__ import annotations

import re
from pathlib import Path
from typing import Iterable, List, Tuple


_PLACEHOLDER_MARKERS = [
    "...",
    "existing code",
    "rest of the code",
    "unchanged",
    "// (rest",
    "// ...",
]


def extract_code_blocks(response: str) -> List[str]:
    """Extract fenced code blocks from an LLM response."""
    if not response:
        return []
    return re.findall(r"```[a-zA-Z0-9_-]*\n([\s\S]*?)\n```", response)


def parse_file_blocks(blocks: Iterable[str]) -> List[Tuple[str, str]]:
    """Parse code blocks into (file_path, content) tuples."""
    parsed: List[Tuple[str, str]] = []
    for block in blocks:
        lines = block.splitlines()
        if not lines:
            continue
        first_line = lines[0].strip()
        if not first_line.startswith("//"):
            continue
        file_path = first_line[2:].strip()
        if not file_path:
            continue
        content = "\n".join(lines[1:]).strip()
        parsed.append((file_path, content))
    return parsed


def has_placeholders(content: str) -> bool:
    lower = content.lower()
    return any(marker in lower for marker in _PLACEHOLDER_MARKERS)


def apply_file_blocks(repo_root: Path, file_blocks: Iterable[Tuple[str, str]]) -> List[str]:
    """Apply parsed file blocks to disk. Returns list of updated files."""
    updated: List[str] = []

    for file_path, content in file_blocks:
        if not content or has_placeholders(content):
            continue

        full_path = repo_root / file_path
        if not full_path.parent.exists():
            continue

        full_path.write_text(content, encoding="utf-8")
        updated.append(file_path)

    return updated
