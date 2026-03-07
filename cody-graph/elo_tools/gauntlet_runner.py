#!/usr/bin/env python3
"""
ELO Gain: Gauntlet Runner

Runs a match between the candidate engine build and the stable baseline using
cutechess-cli with SPRT (Sequential Probability Ratio Test).

Usage:
    python elo_tools/gauntlet_runner.py \
        --candidate <path_to_candidate_binary> \
        --engines-dir <chess_engines_directory> \
        --games 200 \
        --time-control "10+0.1"

Integrates with selfplay.py logic to handle SPRT termination and result analysis.
"""

import argparse
import subprocess
import json
import os
import re
from pathlib import Path
from typing import Dict, Tuple, Optional
import sys

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from elo_tools.version_manager import (
    get_latest_champion_binary,
    copy_candidate_binary
)


DEFAULT_ENGINES_DIR = r"C:\chess\Engines"
DEFAULT_CUTECHESS = r"C:\Program Files (x86)\Cute Chess\cutechess-cli.exe"
DEFAULT_CONCURRENCY = 4
DEFAULT_TIME_CONTROL = "10+0.1"
DEFAULT_MAX_GAMES = 200

# SPRT configuration for detecting +5 ELO improvement
SPRT_ELO0 = 0    # H0: No improvement
SPRT_ELO1 = 5    # H1: +5 ELO improvement
SPRT_ALPHA = 0.05  # Type I error (false positive)
SPRT_BETA = 0.05   # Type II error (false negative)


class GauntletResult:
    """Result of a gauntlet match with detailed outcome information."""
    
    def __init__(self):
        self.status: str = "unknown"  # "pass", "fail", "illegal_move", "timeout", "error"
        self.candidate_wins: int = 0
        self.champion_wins: int = 0
        self.draws: int = 0
        self.games_played: int = 0
        self.candidate_score: float = 0.0
        self.sprt_decision: Optional[str] = None  # "H0" (fail), "H1" (pass), or None
        self.pgn_file: Optional[str] = None
        self.worst_fail_pgn: Optional[str] = None
        self.error_message: Optional[str] = None
        self.illegal_move_by_candidate: bool = False
        self.illegal_move_details: Optional[Dict] = None
    
    def to_dict(self) -> Dict:
        return {
            "status": self.status,
            "candidate_wins": self.candidate_wins,
            "champion_wins": self.champion_wins,
            "draws": self.draws,
            "games_played": self.games_played,
            "candidate_score_percent": self.candidate_score * 100,
            "sprt_decision": self.sprt_decision,
            "pgn_file": self.pgn_file,
            "worst_fail_pgn": self.worst_fail_pgn,
            "error_message": self.error_message,
            "illegal_move_by_candidate": self.illegal_move_by_candidate,
            "illegal_move_details": self.illegal_move_details,
        }


def parse_illegal_move_line(line: str) -> Optional[Dict]:
    """Extract structured illegal-move details from a cutechess output line."""
    if "illegal move" not in line.lower():
        return None

    side_match = re.search(r'(White|Black) makes an illegal move', line, re.IGNORECASE)
    move_match = re.search(r'illegal move:\s*([a-h][1-8][a-h][1-8][nbrqNBRQ]?)', line)
    pairing_match = re.search(r'\(([^()]+)\s+vs\s+([^()]+)\)', line)

    side = side_match.group(1).capitalize() if side_match else None
    move = move_match.group(1).lower() if move_match else None
    white = pairing_match.group(1).strip() if pairing_match else None
    black = pairing_match.group(2).strip() if pairing_match else None

    offender = None
    if side == "White" and white:
        offender = white
    elif side == "Black" and black:
        offender = black

    return {
        "side": side,
        "uci_move": move,
        "white": white,
        "black": black,
        "offender": offender,
        "raw": line.strip(),
    }


def parse_cutechess_output(output: str, result: GauntletResult) -> None:
    """
    Parse cutechess-cli output to extract match statistics and SPRT decision.
    
    Args:
        output: Full stdout/stderr from cutechess-cli
        result: GauntletResult object to populate
    """
    # Parse score line: "Score of Candidate vs Champion: 105 - 83 - 12  [0.555] 200"
    score_match = re.search(
        r'Score of (\w+) vs (\w+):\s*(\d+)\s*-\s*(\d+)\s*-\s*(\d+)\s*\[([^\]]+)\]\s*(\d+)',
        output
    )
    
    if score_match:
        player1 = score_match.group(1)
        wins1 = int(score_match.group(3))
        wins2 = int(score_match.group(4))
        draws = int(score_match.group(5))
        score = float(score_match.group(6))
        games = int(score_match.group(7))
        
        # Assign to result based on player names
        if player1.lower() == "candidate":
            result.candidate_wins = wins1
            result.champion_wins = wins2
        else:
            result.candidate_wins = wins2
            result.champion_wins = wins1
        
        result.draws = draws
        result.games_played = games
        result.candidate_score = score
    
    # Parse SPRT decision
    if "SPRT: H1" in output or "accepted" in output.lower():
        result.sprt_decision = "H1"
        result.status = "pass"
    elif "SPRT: H0" in output or "rejected" in output.lower():
        result.sprt_decision = "H0"
        result.status = "fail"
    
    # Check for illegal moves
    if "illegal move" in output.lower():
        if "Candidate" in output or "candidate" in output:
            result.illegal_move_by_candidate = True
            result.status = "illegal_move"


def parse_worst_fail_pgn(temp_pgn: str, candidate_name: str = "Candidate") -> Optional[str]:
    """
    Parse the PGN file to find and save the worst failure game.
    
    Logic (from selfplay.py):
    1. CRITICAL: Candidate illegal moves (highest priority)
    2. Egregious: Shortest candidate checkmate loss
    3. Other: Shortest candidate loss
    
    Args:
        temp_pgn: Path to temporary PGN file with all games
        candidate_name: Name of candidate engine in PGN
    
    Returns:
        Path to worst_fail.pgn or None if no losses
    """
    if not os.path.exists(temp_pgn):
        return None
    
    with open(temp_pgn, "r") as f:
        content = f.read()
    
    games = re.split(r'\n\n(?=\[Event)', content.strip())
    illegal_candidate_losses = []
    mated_losses = []
    other_losses = []
    
    for game in games:
        if not game.strip():
            continue
        
        white_match = re.search(r'\[White "(.*?)"\]', game)
        black_match = re.search(r'\[Black "(.*?)"\]', game)
        result_match = re.search(r'\[Result "(.*?)"\]', game)
        
        if not (white_match and black_match and result_match):
            continue
        
        white = white_match.group(1)
        black = black_match.group(1)
        result = result_match.group(1)
        
        # Check if Candidate committed the illegal move
        term_match = re.search(r'\{(.*?) makes an illegal move', game)
        illegal_player = term_match.group(1) if term_match else None
        
        is_candidate_illegal = False
        if illegal_player:
            if (illegal_player == "White" and white == candidate_name) or \
               (illegal_player == "Black" and black == candidate_name):
                is_candidate_illegal = True
        
        move_nums = re.findall(r'(\d+)\.', game)
        move_count = int(move_nums[-1]) if move_nums else 999
        
        # Check if candidate lost
        if (white == candidate_name and result == "0-1") or \
           (black == candidate_name and result == "1-0"):
            
            if is_candidate_illegal:
                illegal_candidate_losses.append((move_count, game))
            elif "#" in game or "mate" in game.lower():
                mated_losses.append((move_count, game))
            else:
                other_losses.append((move_count, game))
    
    # Selection Logic
    target_pgn, reason = "", ""
    if illegal_candidate_losses:
        illegal_candidate_losses.sort(key=lambda x: x[0])
        target_pgn, reason = illegal_candidate_losses[0][1], "CRITICAL: Candidate Illegal Move"
    elif mated_losses:
        mated_losses.sort(key=lambda x: x[0])
        target_pgn, reason = mated_losses[0][1], "Egregious Checkmate"
    elif other_losses:
        other_losses.sort(key=lambda x: x[0])
        target_pgn, reason = other_losses[0][1], "Shortest Loss"
    
    if target_pgn:
        worst_fail_path = Path(temp_pgn).parent / "worst_fail.pgn"
        with open(worst_fail_path, "w") as f:
            f.write(target_pgn)
        print(f"\n🚨 {reason} saved to {worst_fail_path}")
        return str(worst_fail_path)
    
    return None


def run_gauntlet(
    candidate_binary: str,
    engines_dir: str = DEFAULT_ENGINES_DIR,
    game_count: int = DEFAULT_MAX_GAMES,
    time_control: str = DEFAULT_TIME_CONTROL,
    cutechess_path: str = DEFAULT_CUTECHESS,
    concurrency: int = DEFAULT_CONCURRENCY,
    output_pgn: Optional[str] = None,
) -> GauntletResult:
    """
    Run gauntlet match between candidate and champion using cutechess-cli with SPRT.
    
    Args:
        candidate_binary: Path to candidate Cody binary
        engines_dir: Directory containing versioned champion binaries
        game_count: Maximum number of games to play (SPRT may terminate early)
        time_control: Time control in "minutes+seconds" format (e.g., "10+0.1")
        cutechess_path: Path to cutechess-cli executable
        concurrency: Number of concurrent games
        output_pgn: Output PGN file path (auto-generated if None)
    
    Returns:
        GauntletResult object with detailed match outcome
    """
    result = GauntletResult()
    
    # Verify cutechess-cli exists
    if not os.path.exists(cutechess_path):
        result.status = "error"
        result.error_message = f"cutechess-cli not found at {cutechess_path}"
        print(f"[gauntlet_runner] ERROR: {result.error_message}")
        return result
    
    # Verify candidate binary exists
    if not os.path.exists(candidate_binary):
        result.status = "error"
        result.error_message = f"Candidate binary not found: {candidate_binary}"
        print(f"[gauntlet_runner] ERROR: {result.error_message}")
        return result
    
    # Copy candidate to engines directory as "cody.exe"
    try:
        candidate_path = copy_candidate_binary(candidate_binary, engines_dir)
    except Exception as e:
        result.status = "error"
        result.error_message = f"Failed to copy candidate binary: {e}"
        print(f"[gauntlet_runner] ERROR: {result.error_message}")
        return result
    
    # Find latest champion
    champion_binary = get_latest_champion_binary(engines_dir)
    if not champion_binary:
        result.status = "error"
        result.error_message = f"No champion binary found in {engines_dir}"
        print(f"[gauntlet_runner] ERROR: {result.error_message}")
        return result
    
    # Setup PGN output
    if output_pgn is None:
        output_pgn = str(Path(engines_dir) / "temp_match.pgn")
    result.pgn_file = output_pgn
    
    # Remove old PGN if exists
    if os.path.exists(output_pgn):
        os.remove(output_pgn)
    
    print(f"[gauntlet_runner] Starting SPRT gauntlet:")
    print(f"  Candidate: {Path(candidate_path).name}")
    print(f"  Champion:  {Path(champion_binary).name}")
    print(f"  Max games: {game_count}")
    print(f"  Time:      {time_control}")
    print(f"  SPRT:      H0=+{SPRT_ELO0} ELO vs H1=+{SPRT_ELO1} ELO")
    
    # Build cutechess-cli command
    cmd = [
        cutechess_path,
        "-engine", f"cmd={candidate_path}", "name=Candidate",
        "-engine", f"cmd={champion_binary}", "name=Champion",
        "-each", f"tc={time_control}", "proto=uci",
        "-sprt", f"elo0={SPRT_ELO0}", f"elo1={SPRT_ELO1}",
                f"alpha={SPRT_ALPHA}", f"beta={SPRT_BETA}",
        "-concurrency", str(concurrency),
        "-rounds", str(game_count),
        "-repeat", "-recover",
        "-pgnout", output_pgn
    ]
    
    # Run match
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
            print(line, end="")
            output_buffer.append(line)

            illegal_info = parse_illegal_move_line(line)
            if illegal_info is not None:
                result.illegal_move_details = illegal_info
            
            # Halt if Candidate made an illegal move
            if "illegal move" in line.lower() and "Candidate" in line:
                if "(Champion vs Candidate): 1-0" in line or \
                   "(Candidate vs Champion): 0-1" in line:
                    print("\n🛑 HALTING: Candidate made an illegal move!")
                    result.illegal_move_by_candidate = True
                    result.status = "illegal_move"
                    process.terminate()
                    break

            # Prefer structured offender detection when available.
            if illegal_info is not None and illegal_info.get("offender") == "Candidate":
                print("\n🛑 HALTING: Candidate made an illegal move!")
                result.illegal_move_by_candidate = True
                result.status = "illegal_move"
                process.terminate()
                break
        
        process.wait()
        
        # Parse output
        full_output = "".join(output_buffer)
        parse_cutechess_output(full_output, result)
        
        # Parse worst failure if there were losses
        if result.candidate_wins < result.champion_wins:
            worst_pgn = parse_worst_fail_pgn(output_pgn, "Candidate")
            if worst_pgn:
                result.worst_fail_pgn = worst_pgn
        
        # Set final status if not already set
        if result.status == "unknown":
            if result.sprt_decision == "H1":
                result.status = "pass"
            elif result.sprt_decision == "H0":
                result.status = "fail"
            else:
                result.status = "timeout"  # Max games reached without SPRT decision
        
        print(f"\n[gauntlet_runner] Match complete:")
        print(f"  Status:    {result.status}")
        print(f"  Candidate: {result.candidate_wins} wins")
        print(f"  Champion:  {result.champion_wins} wins")
        print(f"  Draws:     {result.draws}")
        print(f"  Score:     {result.candidate_score*100:.1f}%")
        print(f"  SPRT:      {result.sprt_decision or 'No decision'}")
        
    except KeyboardInterrupt:
        print("\n[gauntlet_runner] Match interrupted by user")
        process.terminate()
        result.status = "error"
        result.error_message = "Interrupted by user"
    except Exception as e:
        print(f"[gauntlet_runner] ERROR: {e}")
        result.status = "error"
        result.error_message = str(e)
    
    return result


def main():
    parser = argparse.ArgumentParser(
        description="Run SPRT gauntlet match between candidate and champion Cody engines"
    )
    parser.add_argument(
        "--candidate", required=True, help="Path to candidate Cody binary"
    )
    parser.add_argument(
        "--engines-dir", default=DEFAULT_ENGINES_DIR,
        help=f"Engines directory (default: {DEFAULT_ENGINES_DIR})"
    )
    parser.add_argument(
        "--games", type=int, default=DEFAULT_MAX_GAMES, help="Maximum number of games"
    )
    parser.add_argument(
        "--time-control", default=DEFAULT_TIME_CONTROL,
        help=f"Time control (default: {DEFAULT_TIME_CONTROL})"
    )
    parser.add_argument(
        "--cutechess", default=DEFAULT_CUTECHESS,
        help=f"Path to cutechess-cli (default: {DEFAULT_CUTECHESS})"
    )
    parser.add_argument(
        "--concurrency", type=int, default=DEFAULT_CONCURRENCY,
        help=f"Concurrent games (default: {DEFAULT_CONCURRENCY})"
    )
    parser.add_argument(
        "--output", default=None, help="Output PGN file (auto-generated if not specified)"
    )
    
    args = parser.parse_args()
    
    result = run_gauntlet(
        candidate_binary=args.candidate,
        engines_dir=args.engines_dir,
        game_count=args.games,
        time_control=args.time_control,
        cutechess_path=args.cutechess,
        concurrency=args.concurrency,
        output_pgn=args.output,
    )
    
    print(json.dumps(result.to_dict(), indent=2))
    
    # Exit code: 0 for pass, 1 for fail/error
    return 0 if result.status == "pass" else 1


if __name__ == "__main__":
    sys.exit(main())
