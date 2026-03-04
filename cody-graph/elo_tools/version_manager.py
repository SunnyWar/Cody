#!/usr/bin/env python3
"""
Version Manager for ELO Gain Phase

Handles version reading, incrementing, and binary management for the Cody engine.
Versions are stored in engine/Cargo.toml and follow semantic versioning (major.minor.patch).
"""

import re
import shutil
from pathlib import Path
from typing import Tuple, Optional


def read_version(cargo_toml_path: str) -> Tuple[int, int, int]:
    """
    Read version from engine/Cargo.toml.
    
    Returns:
        Tuple of (major, minor, patch) version numbers
    """
    cargo_path = Path(cargo_toml_path)
    if not cargo_path.exists():
        raise FileNotFoundError(f"Cargo.toml not found at {cargo_toml_path}")
    
    content = cargo_path.read_text()
    match = re.search(r'version\s*=\s*"(\d+)\.(\d+)\.(\d+)"', content)
    
    if not match:
        raise ValueError("Could not parse version from Cargo.toml")
    
    return int(match.group(1)), int(match.group(2)), int(match.group(3))


def write_version(cargo_toml_path: str, major: int, minor: int, patch: int) -> None:
    """
    Write version to engine/Cargo.toml.
    
    Args:
        cargo_toml_path: Path to engine/Cargo.toml
        major: Major version number
        minor: Minor version number
        patch: Patch version number
    """
    cargo_path = Path(cargo_toml_path)
    if not cargo_path.exists():
        raise FileNotFoundError(f"Cargo.toml not found at {cargo_toml_path}")
    
    content = cargo_path.read_text()
    new_version = f"{major}.{minor}.{patch}"
    updated_content = re.sub(
        r'version\s*=\s*"\d+\.\d+\.\d+"',
        f'version = "{new_version}"',
        content
    )
    
    cargo_path.write_text(updated_content)
    print(f"[version_manager] Updated version to {new_version}")


def increment_patch(cargo_toml_path: str) -> Tuple[int, int, int]:
    """
    Increment the patch version (least significant).
    Used for: clippy, refactoring, performance, ucifeatures phases.
    
    Args:
        cargo_toml_path: Path to engine/Cargo.toml
    
    Returns:
        New version tuple (major, minor, patch)
    """
    major, minor, patch = read_version(cargo_toml_path)
    new_patch = patch + 1
    write_version(cargo_toml_path, major, minor, new_patch)
    return major, minor, new_patch


def increment_minor(cargo_toml_path: str) -> Tuple[int, int, int]:
    """
    Increment the minor version and reset patch to 0.
    Used for: elogain phase (successful ELO improvements).
    
    Args:
        cargo_toml_path: Path to engine/Cargo.toml
    
    Returns:
        New version tuple (major, minor, patch)
    """
    major, minor, patch = read_version(cargo_toml_path)
    new_minor = minor + 1
    new_patch = 0  # Reset patch when minor increments
    write_version(cargo_toml_path, major, new_minor, new_patch)
    return major, new_minor, new_patch


def get_version_string(cargo_toml_path: str) -> str:
    """
    Get version as a formatted string.
    
    Returns:
        Version string in format "X.Y.Z"
    """
    major, minor, patch = read_version(cargo_toml_path)
    return f"{major}.{minor}.{patch}"


def copy_binary_with_version(
    source_binary: str,
    engines_dir: str,
    version_str: str
) -> str:
    """
    Copy the built binary to the engines directory with version in filename.
    
    Args:
        source_binary: Path to source binary (e.g., target/release/cody.exe)
        engines_dir: Target engines directory (e.g., C:\\chess\\Engines)
        version_str: Version string (e.g., "0.1.5")
    
    Returns:
        Path to the copied versioned binary
    """
    source_path = Path(source_binary)
    if not source_path.exists():
        raise FileNotFoundError(f"Source binary not found: {source_binary}")
    
    engines_path = Path(engines_dir)
    engines_path.mkdir(parents=True, exist_ok=True)
    
    # Determine extension
    ext = source_path.suffix  # e.g., ".exe" on Windows, "" on Linux
    versioned_name = f"cody-v{version_str}{ext}"
    target_path = engines_path / versioned_name
    
    shutil.copy2(source_path, target_path)
    print(f"[version_manager] Copied {source_path.name} → {target_path}")
    
    return str(target_path)


def copy_candidate_binary(
    source_binary: str,
    engines_dir: str
) -> str:
    """
    Copy the candidate binary (without version suffix) to engines directory.
    This is the "cody.exe" that selfplay.py expects.
    
    Args:
        source_binary: Path to source binary (e.g., target/release/cody.exe)
        engines_dir: Target engines directory (e.g., C:\\chess\\Engines)
    
    Returns:
        Path to the copied candidate binary
    """
    source_path = Path(source_binary)
    if not source_path.exists():
        raise FileNotFoundError(f"Source binary not found: {source_binary}")
    
    engines_path = Path(engines_dir)
    engines_path.mkdir(parents=True, exist_ok=True)
    
    # Use the original filename (e.g., "cody.exe")
    target_path = engines_path / source_path.name
    
    shutil.copy2(source_path, target_path)
    print(f"[version_manager] Copied candidate: {source_path.name} → {target_path}")
    
    return str(target_path)


def get_latest_champion_binary(engines_dir: str) -> Optional[str]:
    """
    Find the latest versioned champion binary in the engines directory.
    
    Args:
        engines_dir: Engines directory path
    
    Returns:
        Path to the latest versioned binary, or None if not found
    """
    engines_path = Path(engines_dir)
    if not engines_path.exists():
        return None
    
    pattern = re.compile(r"cody-?v?([\d\.]+)(?:\.exe)?", re.IGNORECASE)
    versioned_files = []
    
    for f in engines_path.iterdir():
        if not f.is_file():
            continue
        match = pattern.match(f.name)
        if match:
            try:
                version_str = match.group(1)
                # Parse as tuple for comparison
                parts = tuple(map(int, version_str.split(".")))
                versioned_files.append((parts, str(f)))
            except:
                continue
    
    if not versioned_files:
        return None
    
    versioned_files.sort(key=lambda x: x[0], reverse=True)
    return versioned_files[0][1]


if __name__ == "__main__":
    # Quick test
    import sys
    
    if len(sys.argv) < 2:
        print("Usage:")
        print("  python version_manager.py read <cargo_toml_path>")
        print("  python version_manager.py increment <cargo_toml_path>")
        sys.exit(1)
    
    command = sys.argv[1]
    
    if command == "read":
        cargo_path = sys.argv[2] if len(sys.argv) > 2 else "engine/Cargo.toml"
        version = get_version_string(cargo_path)
        print(f"Current version: {version}")
    
    elif command == "increment":
        cargo_path = sys.argv[2] if len(sys.argv) > 2 else "engine/Cargo.toml"
        major, minor, patch = increment_patch(cargo_path)
        print(f"New version: {major}.{minor}.{patch}")
    
    else:
        print(f"Unknown command: {command}")
        sys.exit(1)
