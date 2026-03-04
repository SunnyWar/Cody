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
    import argparse
    
    parser = argparse.ArgumentParser(
        description="Cody version management utility",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Increment patch version (for clippy, refactoring, performance, ucifeatures phases)
  python version_manager.py --patch
  
  # Increment minor version (for elogain phase)
  python version_manager.py --minor
  
  # Increment major version (manual only)
  python version_manager.py --major
  
  # Show current version without changes
  python version_manager.py --show
  
  # Use custom Cargo.toml path
  python version_manager.py --show --cargo-path engine/Cargo.toml
        """
    )
    
    parser.add_argument(
        "--patch",
        action="store_true",
        help="Increment patch version (Z in X.Y.Z)"
    )
    parser.add_argument(
        "--minor",
        action="store_true",
        help="Increment minor version and reset patch (Y in X.Y.Z)"
    )
    parser.add_argument(
        "--major",
        action="store_true",
        help="Increment major version and reset minor/patch (X in X.Y.Z)"
    )
    parser.add_argument(
        "--show",
        action="store_true",
        help="Show current version without making changes"
    )
    parser.add_argument(
        "--cargo-path",
        default="engine/Cargo.toml",
        help="Path to engine/Cargo.toml (default: engine/Cargo.toml)"
    )
    
    args = parser.parse_args()
    cargo_path = args.cargo_path
    
    # Show current version
    if args.show:
        current = get_version_string(cargo_path)
        print(f"Current version: {current}")
    
    # Increment patch (Z++)
    elif args.patch:
        major, minor, patch = increment_patch(cargo_path)
        print(f"✓ Patch version bumped: {major}.{minor}.{patch}")
    
    # Increment minor (Y++, Z reset to 0)
    elif args.minor:
        major, minor, patch = increment_minor(cargo_path)
        print(f"✓ Minor version bumped: {major}.{minor}.{patch}")
    
    # Increment major (X++, Y and Z reset to 0)
    elif args.major:
        major, minor, patch = read_version(cargo_path)
        new_major = major + 1
        new_minor = 0
        new_patch = 0
        write_version(cargo_path, new_major, new_minor, new_patch)
        print(f"✓ Major version bumped: {new_major}.{new_minor}.{new_patch}")
    
    else:
        # No action specified, show help
        parser.print_help()
    
    # Exit successfully
    import sys
    sys.exit(0)

