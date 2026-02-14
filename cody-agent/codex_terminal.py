"""
Codex terminal integration helpers.
"""

from __future__ import annotations

import os
import shutil
import subprocess
from datetime import datetime
from pathlib import Path
from typing import Iterable, Optional


def get_codex_model(config: dict) -> Optional[str]:
    """Resolve the Codex model name from config."""
    return config.get("codex_model") or config.get("model")


def _resolve_codex_executable(repo_root: Path, config: dict) -> str:
    override = config.get("codex_path") or os.environ.get("CODEX_PATH")
    if override:
        return override

    resolved = shutil.which("codex")
    if resolved:
        return resolved

    if os.name == "nt":
        candidates = [
            repo_root / "node_modules" / ".bin" / "codex.cmd",
            repo_root / "node_modules" / ".bin" / "codex.exe",
        ]
    else:
        candidates = [repo_root / "node_modules" / ".bin" / "codex"]

    for candidate in candidates:
        if candidate.exists():
            return str(candidate)

    return "codex"


def _iter_config_overrides(config: dict) -> Iterable[str]:
    overrides = config.get("codex_config_overrides")
    if not overrides:
        return []
    if isinstance(overrides, str):
        return [overrides]
    return [str(value) for value in overrides]


def run_codex(prompt: str, config: dict, repo_root: Path, label: str) -> str:
    """Run Codex CLI and return the final message text."""
    logs_dir = repo_root / ".orchestrator_logs"
    logs_dir.mkdir(parents=True, exist_ok=True)
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    output_path = logs_dir / f"{label}_codex_last_message_{timestamp}.txt"

    codex_exe = _resolve_codex_executable(repo_root, config)

    command = [
        codex_exe,
        "exec",
        "--color",
        "never",
        "--cd",
        str(repo_root),
        "--output-last-message",
        str(output_path),
    ]

    model = get_codex_model(config)
    if model:
        command.extend(["--model", model])

    if config.get("use_local"):
        command.append("--oss")
        local_provider = (
            config.get("local_provider")
            or config.get("codex_local_provider")
            or "ollama"
        )
        if local_provider:
            command.extend(["--local-provider", str(local_provider)])

    if config.get("codex_profile"):
        command.extend(["--profile", str(config["codex_profile"])])

    if config.get("codex_skip_git_repo_check"):
        command.append("--skip-git-repo-check")

    if config.get("codex_ephemeral"):
        command.append("--ephemeral")

    for override in _iter_config_overrides(config):
        command.extend(["--config", override])

    result = subprocess.run(
        command,
        input=prompt,
        text=True,
        capture_output=True,
    )

    if result.returncode != 0:
        error_path = logs_dir / f"{label}_codex_error_{timestamp}.txt"
        error_path.write_text(
            """Codex execution failed
Command: {command}

Stdout:
{stdout}

Stderr:
{stderr}
""".format(
                command=" ".join(command),
                stdout=result.stdout,
                stderr=result.stderr,
            ),
            encoding="utf-8",
            errors="replace",
        )
        raise RuntimeError(f"Codex exec failed; see {error_path}")

    if not output_path.exists():
        raise RuntimeError("Codex exec completed without output file")

    return output_path.read_text(encoding="utf-8", errors="replace").strip()
