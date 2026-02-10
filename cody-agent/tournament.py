#!/usr/bin/env python3
import argparse
import random
import subprocess
import tempfile
from pathlib import Path
import os
import shutil

# --- Configuration ---
ENGINE_BIN_NAME = "engine"
CUTECHESS_CMD = r"C:\Program Files (x86)\Cute Chess\cutechess-cli" # Ensure this path is correct
CONCURRENCY = 4  # Number of simultaneous games

def run(cmd, cwd=None):
    print("::", " ".join(cmd))
    subprocess.check_call(cmd, cwd=cwd)

def get_repo_root():
    out = subprocess.check_output(["git", "rev-parse", "--show-toplevel"], text=True)
    return Path(out.strip())

def build_engine(repo_root: Path) -> Path:
    run(["cargo", "build", "--release"], cwd=str(repo_root))
    exe = ENGINE_BIN_NAME + (".exe" if os.name == "nt" else "")
    path = repo_root / "target" / "release" / exe
    if not path.exists():
        raise FileNotFoundError(path)
    return path

def build_baseline_engine(repo_root: Path) -> Path:
    tmp = Path(tempfile.mkdtemp(prefix="baseline_repo_"))
    print(f":: Cloning repo into {tmp}")
    try:
        run(["git", "clone", "--quiet", str(repo_root), str(tmp)])
        run(["git", "checkout", "HEAD"], cwd=str(tmp))
        return build_engine(tmp)
    except Exception as e:
        shutil.rmtree(tmp)
        raise e

def load_epd_lines(epd_path: Path):
    lines = []
    if not epd_path.exists():
        return []
    with open(epd_path, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            parts = line.split()
            if len(parts) < 4:
                continue
            # Construct a standard FEN from the EPD parts
            fen = " ".join(parts[:4]) + " 0 1"
            lines.append(fen)
    return lines

def run_tournament(new_engine: Path, old_engine: Path, epd_path: Path, games: int):
    cmd = [
        CUTECHESS_CMD,
        "-engine", f"name=New_Version", f"cmd={new_engine}", f"dir={new_engine.parent}",
        "-engine", f"name=Baseline", f"cmd={old_engine}", f"dir={old_engine.parent}",
        "-each", "tc=10+0.1", "proto=uci",
        "-games", str(games),
        "-repeat",
        "-concurrency", "1", # Set to 1 for debugging
        "-pgnout", "tournament_results.pgn"
    ]

    # Add opening book/EPD if available
    if epd_path.exists():
        cmd += ["-openings", f"file={epd_path}", "format=epd", "order=random"]

    run(cmd)

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--games", type=int, default=50, help="Number of games to play")
    parser.add_argument("--epd", type=str, help="Path to EPD opening book")
    args = parser.parse_args()

    root = get_repo_root()
    
    print("--- Building Current Version ---")
    new_exe = build_engine(root)
    
    print("--- Building Baseline Version ---")
    old_exe = build_baseline_engine(root)
    
    print(f"--- Starting Tournament ({args.games} games) ---")
    try:
        run_tournament(new_exe, old_exe, Path(args.epd) if args.epd else Path(""), args.games)
    finally:
        # Cleanup baseline temp directory if desired
        if old_exe.exists():
            shutil.rmtree(old_exe.parent.parent.parent)

if __name__ == "__main__":
    main()