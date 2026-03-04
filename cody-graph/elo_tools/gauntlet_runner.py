#!/usr/bin/env python3
"""
ELO Gain: Gauntlet Runner

Runs a match between the candidate engine build and the stable baseline.
Generates PGN output and match statistics.

Usage:
    python elo_tools/gauntlet_runner.py \
        --candidate <path_to_candidate_binary> \
        --stable <path_to_stable_binary> \
        --games 50 \
        --time-control "10+0.1" \
        --output <output_pgn_file>

TODO: Implement full cutechess-cli integration or direct match orchestration.
"""

import argparse
import subprocess
import json
from pathlib import Path
from typing import Dict, Tuple
import sys

def run_gauntlet(
    candidate_binary: str,
    stable_binary: str,
    game_count: int = 50,
    time_control: str = "10+0.1",
    output_pgn: str = "gauntlet_result.pgn",
) -> Dict:
    """
    Run gauntlet match between two engines.
    
    Args:
        candidate_binary: Path to candidate Cody binary
        stable_binary: Path to stable Cody binary
        game_count: Number of games to play
        time_control: Time control in "minutes+seconds" format (e.g., "10+0.1")
        output_pgn: Output PGN file path
    
    Returns:
        Dictionary with match statistics:
        {
            "candidate_wins": int,
            "stable_wins": int,
            "draws": int,
            "candidate_score_percent": float,
            "pgn_file": str,
        }
    
    TODO: Implement using:
    1. cutechess-cli with UCI protocol
    2. or direct UCI communication with both engines
    3. Parse PGN output for game results
    4. Calculate match statistics
    """
    print(f"[gauntlet_runner] Starting gauntlet: {game_count} games at {time_control}")
    print(f"  Candidate: {candidate_binary}")
    print(f"  Stable:    {stable_binary}")
    
    # PLACEHOLDER: Actual implementation will:
    # 1. Verify both binaries exist and are executable
    # 2. Spawn cutechess-cli or equivalent
    # 3. Configure engines, time control, and match settings
    # 4. Parse live match output or final PGN
    # 5. Return comprehensive statistics
    
    # For now, return placeholder structure
    stats = {
        "candidate_wins": 0,
        "stable_wins": 0,
        "draws": 0,
        "candidate_score_percent": 0.0,
        "pgn_file": output_pgn,
        "games_played": game_count,
    }
    
    print(f"[gauntlet_runner] Match complete (PLACEHOLDER)")
    print(f"  Candidate: {stats['candidate_wins']} wins")
    print(f"  Stable:    {stats['stable_wins']} wins")
    print(f"  Draws:     {stats['draws']}")
    print(f"  Score:     {stats['candidate_score_percent']:.1f}%")
    
    return stats

def main():
    parser = argparse.ArgumentParser(
        description="Run gauntlet match between two Cody engines"
    )
    parser.add_argument(
        "--candidate", required=True, help="Path to candidate Cody binary"
    )
    parser.add_argument(
        "--stable", required=True, help="Path to stable Cody binary"
    )
    parser.add_argument(
        "--games", type=int, default=50, help="Number of games to play"
    )
    parser.add_argument(
        "--time-control", default="10+0.1", help="Time control (e.g., '10+0.1')"
    )
    parser.add_argument(
        "--output", default="gauntlet_result.pgn", help="Output PGN file"
    )
    
    args = parser.parse_args()
    
    stats = run_gauntlet(
        candidate_binary=args.candidate,
        stable_binary=args.stable,
        game_count=args.games,
        time_control=args.time_control,
        output_pgn=args.output,
    )
    
    print(json.dumps(stats, indent=2))
    return 0

if __name__ == "__main__":
    sys.exit(main())
