"""
Validation utilities for executors.
Ensures the project always builds before AND after code changes.
"""

import subprocess
import sys
from pathlib import Path
from typing import Optional, Tuple
import json
from console_utils import safe_print
from agent_runner import run_agent
from llm_utils import apply_file_blocks, extract_code_blocks, parse_file_blocks


def run_validation(repo_root: Path) -> bool:
    """Run validate_cargo.py and return True if successful."""
    result = subprocess.run(
        [sys.executable, str(repo_root / "cody-agent" / "validate_cargo.py")],
        cwd=repo_root,
        capture_output=True,
        text=True
    )
    return result.returncode == 0


def get_build_errors(repo_root: Path) -> str:
    """Get cargo build errors as a string."""
    result = subprocess.run(
        ["cargo", "build", "--all-targets", "--all-features"],
        cwd=repo_root,
        capture_output=True,
        text=True
    )
    
    if result.returncode == 0:
        return ""
    
    # Combine stderr and stdout for complete error context
    return f"{result.stderr}\n{result.stdout}".strip()


def fix_build_with_llm(repo_root: Path, config: dict, errors: str) -> bool:
    """
    Ask LLM to fix build errors.
    Returns True if fix was applied and build succeeded.
    """
    system_message = """You are an expert Rust programmer fixing compilation errors.
You will receive cargo build errors and must provide COMPLETE, BUILDABLE file contents to fix them.

CRITICAL RULES:
1. Return ONLY the complete file contents wrapped in ```rust code blocks
2. Include the file path as a comment on the first line: // path/to/file.rs
3. NEVER use placeholders like "...", "existing code", or "unchanged"
4. NEVER omit any code - return the FULL file
5. Make MINIMAL changes - only fix the specific error
6. Preserve all existing functionality

Format your response EXACTLY like this:
```rust
// engine/src/search/core.rs
[COMPLETE FILE CONTENTS HERE - EVERY SINGLE LINE]
```"""

    user_message = f"""The cargo build has failed with these errors:

{errors}

Please provide the COMPLETE fixed file(s) to resolve these errors.
Remember: FULL file contents only, no placeholders or omissions."""

    try:
        llm_response = run_agent(system_message, user_message, config, repo_root, "build_fix", "logic_bugs")

        code_blocks = extract_code_blocks(llm_response)
        file_blocks = parse_file_blocks(code_blocks)
        if not file_blocks:
            safe_print("❌ LLM did not return any code blocks")
            return False

        updated_files = apply_file_blocks(repo_root, file_blocks)
        if not updated_files:
            safe_print("❌ LLM response did not produce valid file updates")
            return False

        for file_path in updated_files:
            safe_print(f"✅ Applied fix to {file_path}")

        return run_validation(repo_root)

    except Exception as e:
        safe_print(f"❌ LLM fix failed: {e}")
        return False


def ensure_builds_or_fix(repo_root: Path, config: dict, stage: str, max_attempts: int = 3) -> bool:
    """
    Ensure the project builds. If not, attempt to fix with LLM.
    
    Args:
        repo_root: Path to the repository root
        config: Configuration dict with LLM settings
        stage: String describing the stage ("pre-change", "post-change")
        max_attempts: Maximum number of fix attempts
    
    Returns:
        True if project builds successfully, False if unable to fix
    """
    if run_validation(repo_root):
        safe_print(f"✅ {stage} validation: Build successful")
        return True
    
    safe_print(f"⚠️ {stage} validation: Build FAILED")
    
    attempts = 0
    while attempts < max_attempts:
        attempts += 1
        safe_print(f"   Attempting fix {attempts}/{max_attempts}...")
        
        errors = get_build_errors(repo_root)
        if not errors:
            # Build succeeded this time
            safe_print(f"✅ Build fixed on attempt {attempts}")
            return True
        
        safe_print(f"   Build errors:\n{errors[:500]}...")
        
        if fix_build_with_llm(repo_root, config, errors):
            safe_print(f"✅ LLM fixed build on attempt {attempts}")
            return True
        else:
            safe_print(f"   Fix attempt {attempts} failed")
    
    safe_print(f"❌ Could not fix build after {max_attempts} attempts")
    return False


def rollback_changes(repo_root: Path, file_paths: list):
    """Roll back changes to specific files."""
    for path in file_paths:
        subprocess.run(
            ["git", "checkout", "HEAD", "--", path],
            cwd=repo_root,
            capture_output=True
        )
    safe_print(f"🔄 Rolled back changes to {len(file_paths)} file(s)")
