import os
import re
import copy
import argparse
import json
import hashlib
import subprocess
import multiprocessing
from datetime import datetime, timezone

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))

# --- CONFIGURATION ---
CUTECHESS_CLI = r"C:\Program Files (x86)\Cute Chess\cutechess-cli.exe" 
# Leave one core free to ensure system stability and reduce search jitter
CONCURRENCY = max(1, multiprocessing.cpu_count() - 1) 
TIME_CONTROL = "10+0.1"
MAX_GAMES = 400         # Higher sample size for better Elo convergence
TEMP_PGN = "temp_match.pgn"
WORST_FAIL_PGN = "worst_fail.pgn"
RUN_MANIFEST = "temp_match.meta.json"

# Reproducibility Settings
OPENING_BOOK = os.path.join(SCRIPT_DIR, "weak.epd")
BOOK_FORMAT = "epd"
BOOK_ORDER = "sequential" 
SEED = 1337
ENGINE_THREADS = 1
ENGINE_HASH_MB = 64
STRICT_OPENING_BOOK = True

PROFILE_PRESETS = {
    "strict": {
        "concurrency": 1,
        "seed": SEED,
        "engine_threads": ENGINE_THREADS,
        "engine_hash_mb": ENGINE_HASH_MB,
        "strict_opening_book": True,
        "book_order": "sequential",
    },
    "fast": {
        "concurrency": CONCURRENCY,
        "seed": None,
        "engine_threads": max(1, multiprocessing.cpu_count() // 2),
        "engine_hash_mb": 256,
        "strict_opening_book": False,
        "book_order": "random",
    },
}


def parse_version_tuple(text):
    try:
        return tuple(int(part) for part in text.split("."))
    except Exception:
        return None


def file_sha256(path):
    hasher = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            hasher.update(chunk)
    return hasher.hexdigest()


def git_head_or_none():
    try:
        result = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            check=True,
            capture_output=True,
            text=True,
        )
        return result.stdout.strip()
    except Exception:
        return None


def cutechess_version_or_none(cutechess_path):
    try:
        result = subprocess.run(
            [cutechess_path, "--version"],
            check=True,
            capture_output=True,
            text=True,
        )
        return result.stdout.strip() or result.stderr.strip() or None
    except Exception:
        return None


def write_run_manifest(cmd, candidate_path, baseline_path, opening_path, cfg):
    manifest = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "cwd": os.getcwd(),
        "command": cmd,
        "settings": {
            "mode": cfg["mode"],
            "time_control": TIME_CONTROL,
            "max_games": MAX_GAMES,
            "concurrency": cfg["concurrency"],
            "seed": cfg["seed"],
            "engine_threads": cfg["engine_threads"],
            "engine_hash_mb": cfg["engine_hash_mb"],
            "opening_book": opening_path,
            "book_format": BOOK_FORMAT,
            "book_order": cfg["book_order"],
            "strict_opening_book": cfg["strict_opening_book"],
        },
        "artifacts": {
            "temp_pgn": os.path.abspath(TEMP_PGN),
            "worst_fail_pgn": os.path.abspath(WORST_FAIL_PGN),
        },
        "engines": {
            "candidate": {
                "path": candidate_path,
                "sha256": file_sha256(candidate_path),
            },
            "baseline": {
                "path": baseline_path,
                "sha256": file_sha256(baseline_path),
            },
        },
        "environment": {
            "git_head": git_head_or_none(),
            "cutechess_version": cutechess_version_or_none(CUTECHESS_CLI),
        },
    }

    with open(RUN_MANIFEST, "w", encoding="utf-8") as f:
        json.dump(manifest, f, indent=2)


def resolve_mode_config(mode):
    cfg = copy.deepcopy(PROFILE_PRESETS[mode])
    cfg["mode"] = mode
    return cfg


def find_matchup_in_dir(directory):
    candidate = "cody.exe"
    candidate_path = os.path.join(directory, candidate)
    if not os.path.exists(candidate_path):
        return None

    # Matches files like cody-v1.2.exe or cody1.0.exe
    pattern = re.compile(r"cody-?v?([\d\.]+)\.exe", re.IGNORECASE)
    versioned_files = []
    for f in os.listdir(directory):
        if f.lower() == candidate.lower(): 
            continue
        match = pattern.match(f)
        if match:
            parsed = parse_version_tuple(match.group(1))
            if parsed is not None:
                versioned_files.append((parsed, f))

    if not versioned_files:
        return None

    # Sort by version to find the most recent baseline
    versioned_files.sort(key=lambda x: x[0], reverse=True)
    baseline = versioned_files[0][1]
    return {
        "engine_dir": directory,
        "candidate_name": candidate,
        "baseline_name": baseline,
        "candidate_path": candidate_path,
        "baseline_path": os.path.join(directory, baseline),
    }


def get_matchup():
    cwd = os.getcwd()
    search_dirs = [
        cwd,
        os.path.join(cwd, "target", "release"),
        os.path.join(cwd, "target", "debug"),
    ]

    for directory in search_dirs:
        if not os.path.isdir(directory):
            continue
        found = find_matchup_in_dir(directory)
        if found is not None:
            return found

    return {
        "engine_dir": None,
        "candidate_name": None,
        "baseline_name": None,
        "candidate_path": None,
        "baseline_path": None,
        "searched_dirs": search_dirs,
    }

def parse_and_save_worst(candidate_name, emergency_pgn=None):
    """
    Scans the match PGN for the Candidate's quickest loss 
    or saves the emergency PGN if an illegal move occurred.
    """
    if emergency_pgn:
        with open(WORST_FAIL_PGN, "w") as f:
            f.write(emergency_pgn)
        print(f"❗ Emergency PGN saved to {WORST_FAIL_PGN}")
        return

    if not os.path.exists(TEMP_PGN):
        return

    with open(TEMP_PGN, "r") as f:
        pgn_data = f.read()

    # Split into individual games
    games = pgn_data.split('[Event')
    worst_game = ""
    shortest_ply = 999

    for game in games:
        if not game.strip(): continue
        game = '[Event' + game
        
        # Check if candidate lost this game
        lost = (f'[White "{candidate_name}"]' in game and "0-1" in game) or \
               (f'[Black "{candidate_name}"]' in game and "1-0" in game)
        
        if lost:
            # Count move numbers to find the shortest game
            move_count = len(re.findall(r'\d+\.', game))
            if move_count < shortest_ply:
                shortest_ply = move_count
                worst_game = game

    if worst_game:
        with open(WORST_FAIL_PGN, "w") as f:
            f.write(worst_game)
        print(f"📉 Shortest loss ({shortest_ply} moves) saved to {WORST_FAIL_PGN}")

def extract_final_score(output_text):
    """Parses Cute Chess output to provide a final verdict on engine performance."""
    score_pattern = re.compile(
        r"Score of (.*?) vs (.*?): (\d+) - (\d+) - (\d+)", re.IGNORECASE
    )
    elo_pattern = re.compile(r"Elo difference: ([\d\.-]+) \+/- ([\d\.]+)")
    
    match = score_pattern.search(output_text)
    elo_match = elo_pattern.search(output_text)
    
    if match:
        name1, name2, win, loss, draw = match.groups()
        win, loss, draw = int(win), int(loss), int(draw)
        total = win + loss + draw
        win_rate = (win + 0.5 * draw) / total if total > 0 else 0
        
        print("\n" + "="*45)
        print("MATCH CONCLUSION")
        print("="*45)
        
        if win > loss:
            print(f"🏆 WINNER: {name1} (Candidate)")
            print(f"Margin: +{win - loss} points")
        elif loss > win:
            print(f"🏆 WINNER: {name2} (Champion)")
            print(f"Margin: +{loss - win} points")
        else:
            print("⚖️ RESULT: Draw (No statistically significant change)")

        print(f"Final Score: {win} - {loss} - {draw}")
        print(f"Win Rate: {win_rate:.2%}")
        
        if elo_match:
            # Elo difference provides a quantified metric of the improvement
            print(f"Elo Diff: {elo_match.group(1)} (±{elo_match.group(2)})")
        print("="*45)

def run_test(cfg):
    # Automatically clean up old PGN data to ensure a fresh benchmark
    if os.path.exists(TEMP_PGN):
        os.remove(TEMP_PGN)

    if not os.path.exists(CUTECHESS_CLI):
        print(f"❌ Error: cutechess-cli not found at '{CUTECHESS_CLI}'.")
        return

    matchup = get_matchup()
    if not matchup["candidate_path"] or not matchup["baseline_path"]:
        print("❌ Error: Could not find a valid engine pair ('cody.exe' + 'cody-v*.exe').")
        searched_dirs = matchup.get("searched_dirs", [os.getcwd()])
        print("   Searched directories:")
        for d in searched_dirs:
            print(f"   - {d}")
        print("   Place both binaries in one folder, for example:")
        print("   - D:\\Cody\\target\\release\\cody.exe")
        print("   - D:\\Cody\\target\\release\\cody-v1.9.3.exe")
        return

    candidate_name = matchup["candidate_name"]
    baseline_name = matchup["baseline_name"]
    candidate_path = os.path.abspath(matchup["candidate_path"])
    baseline_path = os.path.abspath(matchup["baseline_path"])
    opening_path = OPENING_BOOK

    if cfg["strict_opening_book"] and not os.path.exists(opening_path):
        print(f"❌ Error: Opening book '{opening_path}' is missing.")
        print("   Reproducibility mode requires a fixed opening book.")
        return

    cmd = [
        CUTECHESS_CLI,
        "-engine", f"cmd={candidate_path}", "name=Candidate",
        f"option.Threads={cfg['engine_threads']}", f"option.Hash={cfg['engine_hash_mb']}",
        "-engine", f"cmd={baseline_path}", "name=Champion",
        f"option.Threads={cfg['engine_threads']}", f"option.Hash={cfg['engine_hash_mb']}",
        "-each", f"tc={TIME_CONTROL}", "proto=uci",
        "-concurrency", str(cfg["concurrency"]),
        "-rounds", str(MAX_GAMES), 
        "-repeat", 
        "-recover", 
        "-pgnout", TEMP_PGN
    ]

    if cfg["seed"] is not None:
        cmd.extend(["-srand", str(cfg["seed"])])

    # Integrated weak.epd with sequential selection for reproducibility
    if os.path.exists(opening_path):
        cmd.extend([
            "-openings", 
            f"file={opening_path}", 
            f"format={BOOK_FORMAT}", 
            f"order={cfg['book_order']}"
        ])

    write_run_manifest(cmd, candidate_path, baseline_path, opening_path, cfg)

    print(f"--- ⚔️  Testing {candidate_name} vs {baseline_name} ---")
    print(f"--- 📂 Engine dir: {matchup['engine_dir']} ---")
    print(
        "--- ⚙️  "
        f"Mode={cfg['mode']} | Seed={cfg['seed']} | "
        f"Threads={cfg['engine_threads']} | Hash={cfg['engine_hash_mb']}MB | "
        f"Book={opening_path} ({cfg['book_order']}) | Cores={cfg['concurrency']} ---"
    )
    print(f"--- 📝 Run manifest: {RUN_MANIFEST} ---")
    
    process = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    full_output = []
    emergency_pgn = None
    
    try:
        for line in process.stdout:
            print(line, end="")
            full_output.append(line)
            
            # Catch move generation errors immediately
            if "illegal move" in line.lower() and "Candidate" in line:
                print("\n🛑 CRITICAL: Candidate played an illegal move!")
                emergency_pgn = '[Event "Illegal Move Halt"]\n[Result "0-1"]\n\n{Candidate disqualified} 0-1'
                process.terminate()
                break
        process.wait()
    except KeyboardInterrupt:
        process.terminate()
    
    # 1. Analyze and save failure data for debugging
    parse_and_save_worst("Candidate", emergency_pgn)
    
    # 2. Print the quantitative verdict
    extract_final_score("".join(full_output))

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Run Cody selfplay via cutechess-cli.")
    parser.add_argument(
        "--mode",
        choices=sorted(PROFILE_PRESETS.keys()),
        default="strict",
        help="strict = reproducible A/B checks, fast = higher-throughput noisy checks",
    )
    args = parser.parse_args()
    run_test(resolve_mode_config(args.mode))