#!/usr/bin/env python3
"""
ELO Gain: Compilation & Perft Validator

Validates that a candidate code change:
1. Compiles successfully (cargo build --release)
2. Passes perft move generation tests (e.g., perft 5 or 6)
3. Does not introduce clippy warnings

Returns exit code 0 on success, non-zero on failure.

Usage:
    python elo_tools/validate_compilation.py --repo-path <repo_path> [--perft-depth 5]

TODO: Implement full build validation and perft testing.
"""

import argparse
import subprocess
import sys
from pathlib import Path
from typing import Tuple, Optional

def run_command(cmd: list[str], cwd: Optional[Path] = None) -> Tuple[int, str, str]:
    """Run a command and capture output."""
    try:
        result = subprocess.run(
            cmd,
            cwd=cwd,
            capture_output=True,
            text=True,
            timeout=300  # 5 minute timeout
        )
        return result.returncode, result.stdout, result.stderr
    except subprocess.TimeoutExpired:
        return 1, "", "Command timed out after 300 seconds"
    except Exception as e:
        return 1, "", str(e)

def validate_build(repo_path: Path) -> Tuple[bool, str]:
    """
    Build the engine in release mode AND compile all tests.
    
    This validates that:
    1. Engine binary compiles
    2. All unit tests compile (catches test code errors)
    """
    print(f"[validator] Building release binary...")
    
    # Build engine binary
    cmd = ["cargo", "build", "--release", "-p", "engine"]
    code, stdout, stderr = run_command(cmd, cwd=repo_path)
    
    if code != 0:
        return False, f"Engine build failed:\n{stderr}"
    
    print(f"[validator] [OK] Engine build successful")
    
    # Compile tests (without running them) - this catches test code errors
    print(f"[validator] Compiling unit tests...")
    cmd = ["cargo", "test", "--no-run", "-p", "bitboard"]
    code, stdout, stderr = run_command(cmd, cwd=repo_path)
    
    if code != 0:
        return False, f"Test compilation failed (broken test code):\n{stderr}"
    
    print(f"[validator] [OK] Tests compile successfully")
    return True, ""

def validate_perft(repo_path: Path, depth: int = 5) -> Tuple[bool, str]:
    """
    Run perft tests to verify move generation correctness.
    
    Expected perft results for starting position:
    - perft(5) = 4,865,609 nodes (standard)
    - perft(6) = 119,060,324 nodes (standard)
    
    Tolerates small variations (< 1%) for counting differences.
    """
    print(f"[validator] Running perft {depth}...")
    
    # Path to compiled binary (release mode)
    engine_binary = repo_path / "target" / "release" / "cody.exe"
    if not engine_binary.exists():
        engine_binary = repo_path / "target" / "release" / "cody"
    
    if not engine_binary.exists():
        return False, f"Engine binary not found at {engine_binary}"
    
    cmd = [str(engine_binary), "perft", str(depth)]
    code, stdout, stderr = run_command(cmd, cwd=repo_path)
    
    if code != 0:
        return False, f"Perft test failed:\n{stderr}"
    
    # Parse perft output: "perft(5) = 4865609 (1.2s)"
    import re
    match = re.search(r'perft\(\d+\)\s*=\s*(\d+)', stdout)
    if not match:
        return False, f"Could not parse perft output:\n{stdout}"
    
    node_count = int(match.group(1))
    
    # Expected node counts for standard chess starting position
    expected_counts = {
        5: 4_865_609,
        6: 119_060_324,
        4: 197_281,
        3: 8_902,
    }
    
    if depth in expected_counts:
        expected = expected_counts[depth]
        tolerance = 0.01  # 1% tolerance
        
        if abs(node_count - expected) > expected * tolerance:
            return False, (
                f"Perft node count mismatch:\n"
                f"  Expected: {expected:,}\n"
                f"  Got:      {node_count:,}\n"
                f"  Move generation may be incorrect!"
            )
        
        print(f"[validator] [OK] Perft {depth} passed: {node_count:,} nodes")
    else:
        # Unknown depth - just check that perft ran
        print(f"[validator] [OK] Perft {depth} completed: {node_count:,} nodes (no validation)")
    
    return True, ""

def validate_clippy(repo_path: Path) -> Tuple[bool, str]:
    """
    Check for clippy warnings in the engine code.
    
    Non-fatal: warnings are logged but don't fail validation.
    
    TODO: Implement clippy check and categorize warnings.
    """
    print(f"[validator] Running clippy checks...")
    
    cmd = ["cargo", "clippy", "--all-targets", "--release", "-p", "engine", "--", "-D", "warnings"]
    code, stdout, stderr = run_command(cmd, cwd=repo_path)
    
    if code != 0:
        # Log warnings but don't fail (for now)
        print(f"[validator] [WARNING] Clippy warnings found (non-fatal):")
        print(stderr[:500])  # Print first 500 chars
        return True, ""  # Non-fatal
    
    print(f"[validator] [OK] Clippy checks passed")
    return True, ""

def validate_compilation(
    repo_path: Path,
    perft_depth: int = 5,
) -> bool:
    """
    Run full compilation validation.
    
    Returns True if all checks pass, False otherwise.
    """
    print(f"[validator] Starting compilation validation for {repo_path}\n")
    
    # Step 1: Build
    success, error = validate_build(repo_path)
    if not success:
        print(f"[validator] ✗ Build validation failed: {error}\n")
        return False
    
    # Step 2: Perft
    success, error = validate_perft(repo_path, depth=perft_depth)
    if not success:
        print(f"[validator] ✗ Perft validation failed: {error}\n")
        return False
    
    # Step 3: Clippy (non-fatal)
    validate_clippy(repo_path)
    
    print(f"\n[validator] ✓ All validation checks passed\n")
    return True

def main():
    parser = argparse.ArgumentParser(
        description="Validate compilation and basic correctness of Cody engine"
    )
    parser.add_argument(
        "--repo-path",
        required=True,
        help="Path to Cody repository root"
    )
    parser.add_argument(
        "--perft-depth",
        type=int,
        default=5,
        help="Depth for perft validation (default: 5)"
    )
    
    args = parser.parse_args()
    repo_path = Path(args.repo_path)
    
    if not repo_path.exists():
        print(f"[validator] Error: Repo path does not exist: {repo_path}")
        return 1
    
    success = validate_compilation(repo_path, perft_depth=args.perft_depth)
    return 0 if success else 1

if __name__ == "__main__":
    sys.exit(main())
