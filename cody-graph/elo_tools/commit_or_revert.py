#!/usr/bin/env python3
"""
ELO Gain: Commit & Revert Handler

Handles the final step of the ELO Gain loop:
- If ELO Gain > 0: Commit changes and update baseline
- If ELO Gain <= 0: Revert changes and optionally analyze failures

NOTE: This module is now mostly superseded by commit_util.py which handles
version management automatically for all commits.

Usage:
    python elo_tools/commit_or_revert.py \
        --repo-path <repo_path> \
        --elo-gain 25.5 \
        --candidate-description "Implement Null Move Pruning" \
        --pgn <failure_pgn> (if reverting)
"""

import argparse
import subprocess
from pathlib import Path
from typing import Optional
import json
import sys

# Add parent directory to path for commit_util
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
from commit_util import commit_with_version_bump


def run_command(cmd: list[str], cwd: Optional[Path] = None) -> tuple[int, str]:
    """Run a git command and return (exit_code, output)."""
    try:
        result = subprocess.run(
            cmd,
            cwd=cwd,
            capture_output=True,
            text=True,
            timeout=60,
        )
        return result.returncode, result.stdout + result.stderr
    except subprocess.TimeoutExpired:
        return 1, "Command timed out"
    except Exception as e:
        return 1, str(e)


def commit_improvement(
    repo_path: Path,
    description: str,
    elo_gain: float,
) -> bool:
    """
    Commit the improvement to the repository with automatic version bump.
    
    Uses commit_util.commit_with_version_bump() to ensure version is incremented.
    
    Returns True on success, False on failure.
    """
    print(f"[commit_handler] Committing improvement: {description}")
    print(f"  ELO Gain: +{elo_gain:.1f}")
    
    commit_msg = f"{description} (+{elo_gain:.1f} ELO)"
    
    success, new_version, error = commit_with_version_bump(
        repo_path=str(repo_path),
        commit_message=commit_msg,
        phase="ELOGain",
        files_to_add=None  # Add all modified files
    )
    
    if not success:
        print(f"[commit_handler] ERROR: {error}")
        return False
    
    print(f"[commit_handler] ✓ Committed as v{new_version}")
    return True


def revert_candidate(repo_path: Path) -> bool:
    """
    Revert working directory to pre-candidate state.
    
    TODO: Implement revert workflow:
    1. git status to check dirty state
    2. git checkout -- <modified_files> or git reset --hard HEAD
    3. Verify working directory is clean
    
    Returns True on success, False on failure.
    """
    print(f"[commit_handler] Reverting candidate changes...")
    
    # TODO: Implement:
    # 1. git reset --hard HEAD
    # 2. Verify clean state with git status --porcelain
    
    print(f"[commit_handler] PLACEHOLDER: Would revert changes")
    
    return True


def analyze_losses(pgn_path: Path, repo_path: Path) -> Optional[str]:
    """
    Analyze loss PGNs to understand failure modes.
    
    TODO: Implement comprehensive loss analysis:
    1. Parse PGN games where candidate lost
    2. Extract position contexts and move sequences
    3. Identify patterns: time trouble, specific openings, endgame weaknesses
    4. Generate analysis summary for LLM consumption
    
    Returns analysis text or None if no analysis available.
    """
    print(f"[commit_handler] Analyzing failure modes from losses...")
    
    if not pgn_path.exists():
        print(f"[commit_handler] Warning: PGN file not found: {pgn_path}")
        return None
    
    # TODO: Implement robust analysis:
    # - Parse PGN
    # - Identify candidate losses (Result: 0-1 or 1-0 depending on color)
    # - Extract feature positions and move sequences
    # - Clustering: identify patterns (openings, endgames, tactics)
    # - Generate summary: "Candidate struggled in X positions, especially with Y"
    
    analysis = """
[PLACEHOLDER: Failure Analysis]

Candidate engine struggled in the following areas:
- Time management in fast games
- Weak pawn endgames (specific positions where moves were suboptimal)
- Specific opening lines where foundation was poor
- Tactical complications in the middle game

Recommendations for next iteration:
- Consider time management adjustments
- Tune endgame evaluation
- Improve opening book / move ordering
"""
    
    return analysis


def commit_or_revert(
    repo_path: Path,
    elo_gain: float,
    candidate_description: str,
    pgn_path: Optional[Path] = None,
    decision_threshold: float = 0.0,
) -> dict:
    """
    Main decision logic: commit or revert.
    
    Args:
        repo_path: Path to Cody repository
        elo_gain: ELO gain from gauntlet match
        candidate_description: Human-readable description of improvement
        pgn_path: Path to gauntlet PGN (for loss analysis if reverting)
        decision_threshold: ELO gain must exceed this to commit (default 0.0)
    
    Returns:
        {
            "action": "committed" | "reverted",
            "elo_gain": float,
            "message": str,
            "analysis": str or None,
        }
    
    TODO: Implement full decision logic and workflows.
    """
    print(f"\n[commit_handler] ELO Gain Phase - DECISION\n")
    print(f"  ELO Gain: {elo_gain:.1f}")
    print(f"  Threshold: {decision_threshold:.1f}")
    print(f"  Improvement: {candidate_description}\n")
    
    if elo_gain > decision_threshold:
        # COMMIT
        print(f"[commit_handler] ✓ COMMITTING ({elo_gain:.1f} > {decision_threshold:.1f})\n")
        
        success = commit_improvement(repo_path, candidate_description, elo_gain)
        
        return {
            "action": "committed",
            "elo_gain": elo_gain,
            "message": f"Successfully committed: {candidate_description} (+{elo_gain:.1f} ELO)",
            "analysis": None,
        }
    else:
        # REVERT
        print(f"[commit_handler] ✗ REVERTING ({elo_gain:.1f} <= {decision_threshold:.1f})\n")
        
        success = revert_candidate(repo_path)
        analysis = None
        if pgn_path:
            analysis = analyze_losses(pgn_path, repo_path)
        
        return {
            "action": "reverted",
            "elo_gain": elo_gain,
            "message": f"Candidate did not improve ELO, reverted: {candidate_description}",
            "analysis": analysis,
        }


def main():
    parser = argparse.ArgumentParser(
        description="Commit improvement or revert candidate"
    )
    parser.add_argument(
        "--repo-path",
        required=True,
        help="Path to Cody repository"
    )
    parser.add_argument(
        "--elo-gain",
        type=float,
        required=True,
        help="ELO gain from gauntlet"
    )
    parser.add_argument(
        "--candidate-description",
        required=True,
        help="Description of candidate improvement"
    )
    parser.add_argument(
        "--pgn",
        help="Path to gauntlet PGN (for loss analysis if reverting)"
    )
    parser.add_argument(
        "--threshold",
        type=float,
        default=0.0,
        help="ELO threshold for committing (default: 0.0)"
    )
    
    args = parser.parse_args()
    repo_path = Path(args.repo_path)
    pgn_path = Path(args.pgn) if args.pgn else None
    
    result = commit_or_revert(
        repo_path,
        elo_gain=args.elo_gain,
        candidate_description=args.candidate_description,
        pgn_path=pgn_path,
        decision_threshold=args.threshold,
    )
    
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
