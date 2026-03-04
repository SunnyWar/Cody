import os
import re
import subprocess
from packaging import version

# --- CONFIGURATION ---
CUTECHESS_CLI = r"C:\Program Files (x86)\Cute Chess\cutechess-cli.exe" 
CONCURRENCY = 4                 
TIME_CONTROL = "10+0.1"
MAX_GAMES = 200         
TEMP_PGN = "temp_match.pgn"
WORST_FAIL_PGN = "worst_fail.pgn"

def get_matchup():
    current_dir = os.getcwd()
    candidate = "cody.exe"
    if not os.path.exists(os.path.join(current_dir, candidate)):
        return None, None
    pattern = re.compile(r"cody-?v?([\d\.]+)\.exe", re.IGNORECASE)
    versioned_files = []
    for f in os.listdir(current_dir):
        if f.lower() == candidate.lower(): continue
        match = pattern.match(f)
        if match:
            try: versioned_files.append((version.parse(match.group(1)), f))
            except: continue
    if not versioned_files: return None, None
    versioned_files.sort(key=lambda x: x[0], reverse=True)
    return candidate, versioned_files[0][1]

def parse_and_save_worst(candidate_name):
    if not os.path.exists(TEMP_PGN): return
    with open(TEMP_PGN, "r") as f:
        content = f.read()

    games = re.split(r'\n\n(?=\[Event)', content.strip())
    illegal_candidate_losses = []
    mated_losses = []
    other_losses = []

    for game in games:
        if not game.strip(): continue
        white = re.search(r'\[White "(.*?)"\]', game).group(1)
        black = re.search(r'\[Black "(.*?)"\]', game).group(1)
        result = re.search(r'\[Result "(.*?)"\]', game).group(1)
        
        # Check specifically if Candidate committed the illegal move
        # Cutechess format: "0-1 {White makes an illegal move: d2d4}"
        term_match = re.search(r'\{(.*?) makes an illegal move', game)
        illegal_player = term_match.group(1) if term_match else None
        
        is_candidate_illegal = False
        if illegal_player:
            if (illegal_player == "White" and white == candidate_name) or \
               (illegal_player == "Black" and black == candidate_name):
                is_candidate_illegal = True
        
        move_nums = re.findall(r'(\d+)\.', game)
        move_count = int(move_nums[-1]) if move_nums else 999

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
        with open(WORST_FAIL_PGN, "w") as f:
            f.write(target_pgn)
        print(f"\n🚨 {reason} saved to {WORST_FAIL_PGN}")
    
    if os.path.exists(TEMP_PGN): os.remove(TEMP_PGN)

def run_test():
    candidate, baseline = get_matchup()
    if not candidate: return

    cmd = [
        CUTECHESS_CLI,
        "-engine", f"cmd=./{candidate}", "name=Candidate",
        "-engine", f"cmd=./{baseline}", "name=Champion",
        "-each", f"tc={TIME_CONTROL}", "proto=uci",
        "-sprt", "elo0=0", "elo1=5", "alpha=0.05", "beta=0.05",
        "-concurrency", str(CONCURRENCY), "-rounds", str(MAX_GAMES), 
        "-repeat", "-recover", "-pgnout", TEMP_PGN
    ]

    print(f"--- ⚔️  Monitoring {candidate} vs {baseline} ---")
    process = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    
    try:
        for line in process.stdout:
            print(line, end="")
            # HALT ONLY if Candidate is the one who made the illegal move
            # Line format usually: "Finished game 1 (Champion vs Candidate): 1-0 {Black makes an illegal move: e2e4}"
            if "illegal move" in line.lower() and "Candidate" in line:
                # We check if 'Candidate' is the one mentioned as making the move
                # A bit safer to just check if the player who lost is Candidate
                if "(Champion vs Candidate): 1-0" in line or "(Candidate vs Champion): 0-1" in line:
                    print("\n🛑 HALTING: Candidate made an illegal move!")
                    process.terminate()
                    break
        process.wait()
    except KeyboardInterrupt:
        process.terminate()
    
    parse_and_save_worst("Candidate")

if __name__ == "__main__":
    run_test()