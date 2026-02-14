"""
Skill-like agent helpers using the OpenAI Agents SDK.
"""

from __future__ import annotations

from pathlib import Path
from typing import List

from agent_runner import run_agent
from llm_utils import apply_file_blocks, extract_code_blocks, parse_file_blocks


def _load_text(path: Path) -> str:
    if not path.exists():
        return ""
    return path.read_text(encoding="utf-8", errors="replace")


def _skill_inputs(repo_root: Path, config: dict) -> dict:
    skills_config = config.get("skills", {})
    logs_dir = repo_root / ".orchestrator_logs"

    return {
        "ci_log_path": Path(skills_config.get("ci_log_path", logs_dir / "ci_failure.txt")),
        "pr_comments_path": Path(skills_config.get("pr_comments_path", logs_dir / "pr_review_comments.json")),
    }


def _run_skill_agent(label: str, system_prompt: str, user_prompt: str, config: dict, repo_root: Path) -> List[str]:
    response = run_agent(system_prompt, user_prompt, config, repo_root, label)
    blocks = extract_code_blocks(response)
    file_blocks = parse_file_blocks(blocks)
    return apply_file_blocks(repo_root, file_blocks)


def run_github_fix_ci(config: dict, repo_root: Path) -> List[str]:
    inputs = _skill_inputs(repo_root, config)
    ci_log = _load_text(inputs["ci_log_path"]).strip()
    if not ci_log:
        return []

    system_prompt = (
        "You are a senior engineer fixing CI failures. "
        "Return ONLY full file contents in fenced code blocks. "
        "Each block must start with '// path/to/file.ext'."
    )

    user_prompt = (
        "CI failure log:\n\n"
        f"{ci_log}\n\n"
        "Fix the issues by providing full file contents."
    )

    return _run_skill_agent("skill_github_fix_ci", system_prompt, user_prompt, config, repo_root)


def run_github_address_comments(config: dict, repo_root: Path) -> List[str]:
    inputs = _skill_inputs(repo_root, config)
    comments = _load_text(inputs["pr_comments_path"]).strip()
    if not comments:
        return []

    system_prompt = (
        "You are a senior engineer addressing PR review comments. "
        "Return ONLY full file contents in fenced code blocks. "
        "Each block must start with '// path/to/file.ext'."
    )

    user_prompt = (
        "PR review comments:\n\n"
        f"{comments}\n\n"
        "Apply the requested updates by providing full file contents."
    )

    return _run_skill_agent("skill_github_address_comments", system_prompt, user_prompt, config, repo_root)
