"""
OpenAI Agents SDK runner helpers.
"""

from __future__ import annotations

from datetime import datetime
from pathlib import Path
from typing import Optional

import os

from agents import Agent, Runner


def _resolve_model(config: dict, task_type: str) -> str | None:
    models = config.get("models", {})
    if isinstance(models, dict):
        model = models.get(task_type)
        if model:
            return model
    return config.get("model")


def run_agent(
    system_prompt: str,
    user_prompt: str,
    config: dict,
    repo_root: Path,
    label: str,
    task_type: str,
) -> str:
    """Run an agent synchronously and return the final output."""
    if not os.environ.get("OPENAI_API_KEY"):
        codex_key = os.environ.get("CODEX_API_KEY")
        if codex_key:
            os.environ["OPENAI_API_KEY"] = codex_key

    model = _resolve_model(config, task_type)
    instructions = system_prompt or "You are a helpful assistant."

    agent = Agent(
        name=label,
        instructions=instructions,
        model=model,
    )

    result = Runner.run_sync(agent, user_prompt)
    output = (result.final_output or "").strip()

    logs_dir = repo_root / ".orchestrator_logs"
    logs_dir.mkdir(parents=True, exist_ok=True)
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    output_path = logs_dir / f"{label}_agent_output_{timestamp}.txt"
    output_path.write_text(output, encoding="utf-8", errors="replace")

    return output
