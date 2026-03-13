import subprocess
import sys
import json

# CONFIGURATION
import os
CUTECHESS_CLI = r"C:\Program Files (x86)\Cute Chess\cutechess-cli.exe"
ENGINE_PATH = sys.argv[1] if len(sys.argv) > 1 else "../target/release/cody.exe"
GAMES = int(sys.argv[2]) if len(sys.argv) > 2 else 5
RESULTS_FILE = sys.argv[3] if len(sys.argv) > 3 else "results.json"

# Baseline engine (for tuning, can be same as candidate or a reference)
BASELINE_PATH = sys.argv[4] if len(sys.argv) > 4 else ENGINE_PATH

# Check engine files exist

# Copy engine binaries to current directory for CuteChess
import shutil
def normalize_path(p):
    return os.path.normcase(os.path.abspath(p))
local_candidate = normalize_path(os.path.join(os.getcwd(), os.path.basename(ENGINE_PATH)))
# Always copy baseline from build release location
build_baseline = normalize_path(os.path.join(os.path.dirname(__file__), '..', 'target', 'release', 'cody.exe'))
local_baseline = normalize_path(os.path.join(os.getcwd(), 'baseline.exe'))
import time
def safe_copy(src, dst, label):
    max_attempts = 5
    for attempt in range(max_attempts):
        try:
            if os.path.exists(dst):
                try:
                    os.remove(dst)
                except Exception as e:
                    print(f"WARNING: Could not remove {label} ({dst}): {e}")
                    time.sleep(1)
                    continue
            shutil.copy2(src, dst)
            return True
        except Exception as e:
            print(f"WARNING: Could not copy {label} ({src} -> {dst}): {e}")
            time.sleep(1)
    print(f"ERROR: Could not copy {label} after {max_attempts} attempts.")
    return False

if not safe_copy(ENGINE_PATH, local_candidate, "candidate engine"):
    sys.exit(1)
# Always copy D:/cody/target/release/cody.exe to baseline.exe before the match
try:
    shutil.copy2(build_baseline, local_baseline)
    print(f"Copied {build_baseline} to {local_baseline} as baseline engine.")
except Exception as e:
    print(f"ERROR: Failed to copy {build_baseline} to {local_baseline}: {e}")
    sys.exit(1)

# Run cutechess match

cmd = [
    CUTECHESS_CLI,
    "-engine", f"cmd=./{os.path.basename(ENGINE_PATH)}", "name=Candidate",
    "-engine", f"cmd=./{os.path.basename(BASELINE_PATH)}", "name=Champion",
    "-each", "tc=10+0.1", "proto=uci",
    "-games", str(GAMES),
    "-openings", "file=UHO_Lichess_4852_v1.epd", "format=epd", "order=sequential",
    "-repeat", "-recover",
    "-pgnout", "temp_match.pgn"
]


print("Running CuteChess command:")
print(" ", " ".join(cmd))
result = subprocess.run(cmd, capture_output=True, text=True)
print("CuteChess stdout:")
print(result.stdout)
print("CuteChess stderr:")
print(result.stderr)
if result.returncode != 0:
    print("\n==================== CuteChess ERROR ====================")
    print(f"CRITICAL: CuteChess failed with exit code {result.returncode}")
    print("STDOUT:")
    print(result.stdout)
    print("STDERR:")
    print(result.stderr)
    print("========================================================\n")
    sys.exit(1)

import os
if not os.path.exists("temp_match.pgn"):
    print("WARNING: temp_match.pgn was not created. Check engine paths and CuteChess output above.")

# Parse PGN for win/loss/draw counts (dummy logic, replace with actual parsing)
# Replace the "dummy logic" in selfplay.py with this:
wins = 0
losses = 0
draws = 0

if os.path.exists("temp_match.pgn"):
    with open("temp_match.pgn", 'r') as f:
        pgn_content = f.read()
        # CuteChess outputs results as [Result "1-0"], [Result "0-1"], etc.
        wins = pgn_content.count('[Result "1-0"]')
        losses = pgn_content.count('[Result "0-1"]')
        draws = pgn_content.count('[Result "1/2-1/2"]')

# If no games were played, avoid writing 0/0/0 which might break the optimizer
if wins + losses + draws == 0:
    print("CRITICAL: No games were parsed from PGN!")
    sys.exit(1)

with open(RESULTS_FILE, 'w', encoding='utf-8') as f:
    json.dump({"wins": wins, "losses": losses, "draws": draws}, f)
print(f"Self-play results written to {RESULTS_FILE}")