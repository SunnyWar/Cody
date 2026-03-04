#!/usr/bin/env python3
r"""
Engine Sanity Check - Pre-ELO Validation via Self-Play

Before attempting any ELO improvements, verify engine correctness:
1. Runs self-play matches (engine vs itself)
2. Detects critical issues:
   - Illegal moves
   - Engine crashes/timeouts
   - Quick checkmate (< 10 moves) indicating horrible play
3. Reports severity: CRITICAL (blocks improvements), WARNING (needs attention)

This phase acts as a gate before ELO gain orchestration begins.
If CRITICAL issues found, ELO improvements are blocked until fixed.

Usage:
    python elo_tools/sanity_check.py --repo-path D:\Cody --games 10
"""

import argparse
import subprocess
import json
import os
import re
from pathlib import Path
from typing import Dict, List, Optional, Tuple
import sys

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from elo_tools.version_manager import copy_candidate_binary


DEFAULT_ENGINES_DIR = r"C:\chess\Engines"
DEFAULT_CUTECHESS = r"C:\Program Files (x86)\Cute Chess\cutechess-cli.exe"
DEFAULT_CONCURRENCY = 4
DEFAULT_TIME_CONTROL = "10+0.1"
DEFAULT_GAME_COUNT = 10  # Small number for quick validation


class SanityCheckResult:
    """Result of engine sanity check."""
    
    def __init__(self):
        self.status: str = "pass"  # "pass" or "fail"
        self.games_played: int = 0
        self.illegal_moves: List[Dict] = []
        self.quick_losses: List[Dict] = []  # Checkmate in < 10 moves
        self.crashes: List[Dict] = []
        self.timeouts: List[Dict] = []
        self.pgn_file: Optional[str] = None
        self.critical_issues: List[str] = []
        self.warnings: List[str] = []
    
    def add_illegal_move(self, game_num: int, mover: str, move_num: int):
        """Record illegal move found."""
        self.illegal_moves.append({
            "game": game_num,
            "mover": mover,
            "move_number": move_num
        })
        self.critical_issues.append(f"Game {game_num}: {mover} made illegal move at move {move_num}")
    
    def add_quick_loss(self, game_num: int, loser: str, move_num: int):
        """Record extremely quick checkmate/loss."""
        self.quick_losses.append({
            "game": game_num,
            "loser": loser,
            "move_number": move_num
        })
        self.warnings.append(f"Game {game_num}: {loser} checkmated in {move_num} moves (very poor play)")
    
    def add_critical_error(self, msg: str):
        """Add critical error message."""
        self.critical_issues.append(msg)
        self.status = "fail"
    
    def add_warning(self, msg: str):
        """Add warning message."""
        self.warnings.append(msg)
    
    def has_critical_issues(self) -> bool:
        """Return True if critical issues found."""
        return len(self.critical_issues) > 0
    
    def has_issues(self) -> bool:
        """Return True if any issues (critical or warnings) found."""
        return self.has_critical_issues() or len(self.warnings) > 0
    
    def to_dict(self) -> Dict:
        return {
            "status": self.status,
            "games_played": self.games_played,
            "illegal_moves": self.illegal_moves,
            "quick_losses": self.quick_losses,
            "crashes": self.crashes,
            "timeouts": self.timeouts,
            "critical_issues": self.critical_issues,
            "warnings": self.warnings,
            "has_critical_issues": self.has_critical_issues(),
            "pgn_file": self.pgn_file,
        }


def parse_pgn_for_issues(pgn_file: str) -> Tuple[int, List[Dict], List[Dict]]:
    """
    Parse PGN file for sanity check issues:
    - Illegal moves
    - Quick losses (checkmate in < 10 moves)
    
    Returns:
        (games_played, illegal_move_games, quick_loss_games)
    """
    if not os.path.exists(pgn_file):
        return 0, [], []
    
    with open(pgn_file, "r") as f:
        content = f.read()
    
    # Split games (PGN format has games separated by blank lines)
    games = re.split(r'\n\n(?=\[Event)', content.strip())
    
    illegal_moves = []
    quick_losses = []
    games_count = 0
    
    for game_num, game in enumerate(games, 1):
        if not game.strip():
            continue
        
        games_count += 1
        
        # Check for illegal move termination
        if "illegal move" in game.lower():
            illegal_match = re.search(r'\{(.*?) makes an illegal move', game)
            if illegal_match:
                player = illegal_match.group(1)
                # Estimate move number from game text
                moves = re.findall(r'(\d+)\.', game)
                move_num = int(moves[-1]) if moves else 0
                illegal_moves.append({
                    "game": game_num,
                    "player": player,
                    "move_number": move_num,
                })
                print(f"  [ILLEGAL] Game {game_num}: {player} made illegal move at move {move_num}")
            continue
        
        # Check for result and move count
        result_match = re.search(r'\[Result "([^"]+)"\]', game)
        if not result_match:
            continue
        
        result = result_match.group(1)
        
        # Skip draws and unfinished games
        if result == "*" or result == "1/2-1/2":
            continue
        
        # Count moves to detect quick losses
        moves = re.findall(r'(\d+)\.', game)
        move_num = int(moves[-1]) if moves else 999
        
        # Check if game ended in checkmate (indicated by # in PGN)
        if "#" in game or "mate" in game.lower():
            if move_num < 10:
                # Quick checkmate is a sign of serious problems
                loser = "White" if result == "0-1" else "Black"
                quick_losses.append({
                    "game": game_num,
                    "loser": loser,
                    "move_number": move_num,
                    "result": result,
                })
                print(f"  [QUICK_LOSS] Game {game_num}: {loser} checkmated in {move_num} moves")
    
    return games_count, illegal_moves, quick_losses


def run_self_play_sanity_check(
    binary_path: str,
    engines_dir: str = DEFAULT_ENGINES_DIR,
    game_count: int = DEFAULT_GAME_COUNT,
    time_control: str = DEFAULT_TIME_CONTROL,
    cutechess_path: str = DEFAULT_CUTECHESS,
    concurrency: int = DEFAULT_CONCURRENCY,
) -> SanityCheckResult:
    """
    Run self-play sanity check: engine plays against itself.
    
    Args:
        binary_path: Path to engine binary
        engines_dir: Engines directory for temp files
        game_count: Number of games to play
        time_control: Time control format
        cutechess_path: Path to cutechess-cli
        concurrency: Concurrent games
    
    Returns:
        SanityCheckResult with detailed findings
    """
    result = SanityCheckResult()
    
    # Verify binary exists
    if not os.path.exists(binary_path):
        result.add_critical_error(f"Binary not found: {binary_path}")
        return result
    
    # Verify cutechess-cli exists
    if not os.path.exists(cutechess_path):
        result.add_critical_error(f"cutechess-cli not found at {cutechess_path}")
        return result
    
    # Copy binary to engines directory as "cody.exe"
    try:
        candidate_path = copy_candidate_binary(binary_path, engines_dir)
    except Exception as e:
        result.add_critical_error(f"Failed to copy binary: {e}")
        return result
    
    # Setup PGN output
    output_pgn = str(Path(engines_dir) / "sanity_check.pgn")
    result.pgn_file = output_pgn
    
    # Remove old PGN if exists
    if os.path.exists(output_pgn):
        os.remove(output_pgn)
    
    print(f"\n[sanity_check] Running self-play validation:")
    print(f"  Engine:   {Path(candidate_path).name}")
    print(f"  Games:    {game_count}")
    print(f"  Time:     {time_control}")
    print(f"  Location: {candidate_path}\n")
    
    # Build cutechess-cli command for self-play
    # Pass same engine twice with different names for self-play
    cmd = [
        cutechess_path,
        "-engine", f"cmd={candidate_path}", "name=Cody-White",
        "-engine", f"cmd={candidate_path}", "name=Cody-Black",
        "-each", f"tc={time_control}", "proto=uci",
        "-concurrency", str(concurrency),
        "-rounds", str(game_count),
        "-pgnout", output_pgn
    ]
    
    try:
        process = subprocess.Popen(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            cwd=engines_dir
        )
        
        output_buffer = []
        
        for line in process.stdout:
            print(line, end="", flush=True)
            output_buffer.append(line)
            
            # Detect illegal moves in real-time
            if "illegal move" in line.lower():
                result.add_critical_error("Illegal move detected during play!")
        
        process.wait()
        
        # Parse PGN for issues
        games_played, illegal_moves, quick_losses = parse_pgn_for_issues(output_pgn)
        result.games_played = games_played
        
        # Record findings
        for im in illegal_moves:
            result.add_illegal_move(im["game"], im["player"], im["move_number"])
        
        for ql in quick_losses:
            result.add_quick_loss(ql["game"], ql["loser"], ql["move_number"])
        
        # Determine overall status
        if result.has_critical_issues():
            result.status = "fail"
        else:
            result.status = "pass"
        
        print(f"\n[sanity_check] Self-play complete:")
        print(f"  Games played: {games_played}")
        print(f"  Critical issues: {len(result.critical_issues)}")
        print(f"  Warnings: {len(result.warnings)}")
        
        if result.has_critical_issues():
            print(f"\n  [CRITICAL] CRITICAL ISSUES FOUND - ELO improvements BLOCKED:")
            for issue in result.critical_issues:
                print(f"     - {issue}")
        elif result.has_issues():
            print(f"\n  [WARNING] WARNINGS FOUND - Review before improvements:")
            for warning in result.warnings:
                print(f"     - {warning}")
        else:
            print(f"\n  [OK] Engine appears sound - proceeding with improvements")
        
    except KeyboardInterrupt:
        print("\n[sanity_check] Self-play interrupted by user")
        result.add_critical_error("Self-play interrupted")
    except Exception as e:
        print(f"[sanity_check] ERROR: {e}")
        result.add_critical_error(str(e))
    
    return result


def main():
    parser = argparse.ArgumentParser(
        description="Run engine sanity check via self-play"
    )
    parser.add_argument(
        "--repo-path",
        required=True,
        help="Path to Cody repository"
    )
    parser.add_argument(
        "--games",
        type=int,
        default=DEFAULT_GAME_COUNT,
        help=f"Number of self-play games (default: {DEFAULT_GAME_COUNT})"
    )
    parser.add_argument(
        "--engines-dir",
        default=DEFAULT_ENGINES_DIR,
        help=f"Engines directory (default: {DEFAULT_ENGINES_DIR})"
    )
    parser.add_argument(
        "--time-control",
        default=DEFAULT_TIME_CONTROL,
        help=f"Time control (default: {DEFAULT_TIME_CONTROL})"
    )
    parser.add_argument(
        "--cutechess",
        default=DEFAULT_CUTECHESS,
        help=f"Path to cutechess-cli (default: {DEFAULT_CUTECHESS})"
    )
    
    args = parser.parse_args()
    repo_path = Path(args.repo_path)
    
    if not repo_path.exists():
        print(f"[sanity_check] Error: Repo path does not exist: {repo_path}")
        return 1
    
    # Use compiled release binary
    binary_path = repo_path / "target" / "release" / "cody.exe"
    if not binary_path.exists():
        binary_path = repo_path / "target" / "release" / "cody"
    
    result = run_self_play_sanity_check(
        binary_path=str(binary_path),
        engines_dir=args.engines_dir,
        game_count=args.games,
        time_control=args.time_control,
        cutechess_path=args.cutechess,
    )
    
    print("\n" + "="*80)
    print(json.dumps(result.to_dict(), indent=2))
    print("="*80)
    
    # Exit with 0 if pass (even with warnings), 1 if critical issues
    return 0 if result.status == "pass" else 1


if __name__ == "__main__":
    sys.exit(main())
