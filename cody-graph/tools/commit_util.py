#!/usr/bin/env python3
"""
Commit Utility - Centralized version management for Rust code commits

This utility ensures version increments ONLY when Rust (.rs) files are modified.
Only Rust code changes affect the chess engine's gameplay behavior.

Usage:
    from commit_util import commit_with_version_bump
    
    # Version incremented if .rs files changed
    commit_with_version_bump(
        repo_path=".",
        commit_message="Fix clippy warnings",
        phase="clippy"
    )
"""

import os
import subprocess
import sys
from pathlib import Path
from typing import Optional

# Add elo_tools to path so version_manager can be imported
# (This uses dynamic path insertion for runtime execution, which is necessary
# for this module to work when imported from different locations)
sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "elo_tools"))
from version_manager import increment_patch, increment_minor, get_version_string  # type: ignore


def has_rust_changes(repo_path: str, files_to_add: Optional[list[str]] = None) -> bool:
    """
    Check if any Rust (.rs) files have changed.
    
    Args:
        repo_path: Path to repository root
        files_to_add: Specific files being added, or None to check all modified files
    
    Returns:
        True if any .rs files are modified
    """
    if files_to_add:
        # Check specific files
        return any(f.endswith('.rs') for f in files_to_add)
    
    # Check all modified and staged files
    result = subprocess.run(
        ["git", "diff", "--cached", "--name-only"],
        cwd=repo_path,
        capture_output=True,
        text=True
    )
    
    if result.returncode == 0:
        staged_files = result.stdout.strip().split('\n')
        if any(f.endswith('.rs') for f in staged_files if f):
            return True
    
    # Check unstaged modifications
    result = subprocess.run(
        ["git", "diff", "--name-only"],
        cwd=repo_path,
        capture_output=True,
        text=True
    )
    
    if result.returncode == 0:
        modified_files = result.stdout.strip().split('\n')
        if any(f.endswith('.rs') for f in modified_files if f):
            return True
    
    return False


def commit_with_version_bump(
    repo_path: str,
    commit_message: str,
    phase: str = "general",
    files_to_add: Optional[list[str]] = None,
) -> tuple[bool, str, str]:
    """
    Commit changes with automatic version increment (only if .rs files changed).
    
    Version is ONLY incremented if Rust (.rs) source files are modified,
    since only Rust code changes affect engine gameplay behavior.
    
    Args:
        repo_path: Path to repository root
        commit_message: Commit message (version will be prepended if .rs changed)
        phase: Phase name for logging (e.g., "clippy", "refactoring", "elogain")
        files_to_add: Specific files to add, or None to add all modified files
    
    Returns:
        Tuple of (success: bool, version: str, error_message: str)
    """
    try:
        # Check if any Rust files are being changed
        rust_changed = has_rust_changes(repo_path, files_to_add)
        
        current_version = get_version_string(os.path.join(repo_path, "engine", "Cargo.toml"))
        should_bump_version = rust_changed
        
        # 1. Conditionally increment version if .rs files changed
        cargo_toml = os.path.join(repo_path, "engine", "Cargo.toml")
        if should_bump_version:
            # ELOGain phase increments MINOR version (Y), others increment PATCH (Z)
            if phase.lower() == "elogain":
                major, minor, patch = increment_minor(cargo_toml)
                print(f"[commit_util] Minor version bumped: {major}.{minor}.{patch} (ELOGain phase)", flush=True)
            else:
                major, minor, patch = increment_patch(cargo_toml)
                print(f"[commit_util] Patch version bumped: {major}.{minor}.{patch} ({phase} phase)", flush=True)
            new_version = f"{major}.{minor}.{patch}"
        else:
            new_version = current_version
            print(f"[commit_util] No version bump: {new_version} (no Rust files changed)", flush=True)
        
        # 2. Stage files for commit
        if files_to_add:
            # Add specific files
            for file_path in files_to_add:
                result = subprocess.run(
                    ["git", "add", file_path],
                    cwd=repo_path,
                    capture_output=True,
                    text=True
                )
                if result.returncode != 0:
                    return False, new_version, f"Failed to add {file_path}: {result.stderr}"
        else:
            # Add all modified files (excluding untracked)
            result = subprocess.run(
                ["git", "add", "-u"],
                cwd=repo_path,
                capture_output=True,
                text=True
            )
            if result.returncode != 0:
                return False, new_version, f"Failed to stage files: {result.stderr}"
        
        # Add Cargo.toml only if version changed
        if should_bump_version:
            result = subprocess.run(
                ["git", "add", "engine/Cargo.toml"],
                cwd=repo_path,
                capture_output=True,
                text=True
            )
            if result.returncode != 0:
                return False, new_version, f"Failed to add Cargo.toml: {result.stderr}"
        
        # 3. Create commit with version prefix only if version bumped
        if should_bump_version:
            full_message = f"v{new_version} - {phase}: {commit_message}"
        else:
            full_message = f"{phase}: {commit_message}"
        
        result = subprocess.run(
            ["git", "commit", "-m", full_message],
            cwd=repo_path,
            capture_output=True,
            text=True
        )
        
        if result.returncode != 0:
            # Check if there were no changes to commit
            if "nothing to commit" in result.stdout.lower() or "nothing to commit" in result.stderr.lower():
                print(f"[commit_util] No changes to commit", flush=True)
                return True, new_version, "No changes"
            return False, new_version, f"Failed to commit: {result.stderr}"
        
        print(f"[commit_util] Committed: {full_message}", flush=True)
        print(f"[commit_util] Commit hash: {result.stdout.strip()}", flush=True)
        
        return True, new_version, ""
        
    except Exception as e:
        error_msg = f"Exception during commit: {str(e)}"
        print(f"[commit_util] ERROR: {error_msg}", flush=True)
        return False, "", error_msg


def get_current_version(repo_path: str) -> str:
    """
    Get the current version from Cargo.toml.
    
    Args:
        repo_path: Path to repository root
    
    Returns:
        Version string (e.g., "0.1.5")
    """
    cargo_toml = os.path.join(repo_path, "engine", "Cargo.toml")
    return get_version_string(cargo_toml)


if __name__ == "__main__":
    # Quick test
    import argparse
    
    parser = argparse.ArgumentParser(description="Commit with automatic version bump")
    parser.add_argument("--repo", default=".", help="Repository path")
    parser.add_argument("--message", required=True, help="Commit message")
    parser.add_argument("--phase", default="manual", help="Phase name")
    parser.add_argument("--files", nargs="*", help="Specific files to add")
    
    args = parser.parse_args()
    
    success, version, error = commit_with_version_bump(
        repo_path=args.repo,
        commit_message=args.message,
        phase=args.phase,
        files_to_add=args.files
    )
    
    if success:
        print(f"✓ Successfully committed as v{version}")
        sys.exit(0)
    else:
        print(f"✗ Commit failed: {error}")
        sys.exit(1)
