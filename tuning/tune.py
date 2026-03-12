"""
Master orchestration script for Bayesian Tuning.
1. Establishes a static 'Gold' baseline.
2. Iteratively suggests parameters, builds, and tests against that baseline.
"""

import subprocess
import os
import shutil
import sys
import json

CONFIG_FILE = os.path.join(os.path.dirname(__file__), "tuning_config.json")
with open(CONFIG_FILE, "r") as f:
    config = json.load(f)

TUNING_DIR = config["tuning_dir"]
ENGINE_SRC_BIN = config["engine_bin"]
BASELINE_BIN = config["baseline_bin"]
PARAMS_FILE = "params.json"
RESULTS_FILE = "results.json"
CODY_TUNER_EXE = config["cody_tuner_exe"]
MAX_ITERATIONS = config["max_iterations"]
GAMES_PER_MATCH = config["games_per_match"]

def run_step(cmd, description, cwd=None):
    print(f"\n>>> Step: {description}")
    # Using shell=True can be helpful for Windows paths/python calls
    if cwd is None:
        cwd = TUNING_DIR
    result = subprocess.run(cmd, cwd=cwd, shell=True, capture_output=True, text=True)
    if result.returncode != 0:
        print(f"FAILED: {description}")
        if result.stderr:
            print("Error output:")
            print(result.stderr)
        sys.exit(1)
    return result.stdout if result.stdout else None

def main():
    # Reset tuner history to ensure a clean run
    CODYTUNER_DIR = os.path.dirname(CODY_TUNER_EXE)
    run_step([CODY_TUNER_EXE, "reset"], "Resetting tuner history", cwd=CODYTUNER_DIR)
    # Ensure we are starting fresh in the tuning directory
    if not os.path.exists(TUNING_DIR):
        os.makedirs(TUNING_DIR)

    # 1. Initialize params.json from the current source code
    run_step(["python", "edit_consts.py", "--extract"], "Extracting initial constants from source")

    # 2. Build the initial 'Gold' version to serve as the benchmark
    run_step(["python", "build_engine.py"], "Building the initial Gold baseline")

    # 3. Copy the build to a static location so it isn't overwritten by the tuner
    if not os.path.exists(ENGINE_SRC_BIN):
        print(f"ERROR: Build output not found at {ENGINE_SRC_BIN}")
        sys.exit(1)
    
    shutil.copy2(ENGINE_SRC_BIN, BASELINE_BIN)
    print(f"SUCCESS: Static baseline established at {BASELINE_BIN}")

    # 4. Initialize results.json with dummy data if it doesn't exist
    # This prevents the optimizer from crashing on the very first run.
    if not os.path.exists(os.path.join(TUNING_DIR, RESULTS_FILE)):
        with open(os.path.join(TUNING_DIR, RESULTS_FILE), 'w') as f:
            json.dump({"wins": 0, "losses": 0, "draws": 0, "score": 0}, f)

    # 5. Tuning Loop
    for i in range(MAX_ITERATIONS):
        print(f"\n{'='*40}")
        print(f" STARTING ITERATION {i+1} / {MAX_ITERATIONS}")
        print(f"{'='*40}")

        # A. Sync params.json and tuner_history.json to cody_tuner.exe directory
        CODYTUNER_DIR = os.path.dirname(CODY_TUNER_EXE)
        tuner_params_path = os.path.join(CODYTUNER_DIR, "params.json")
        tuner_history_path = os.path.join(CODYTUNER_DIR, "tuner_history.json")
        local_params_path = os.path.join(TUNING_DIR, "params.json")
        local_history_path = os.path.join(TUNING_DIR, "tuner_history.json")
        # Copy params.json
        shutil.copy2(local_params_path, tuner_params_path)
        # Copy tuner_history.json if it exists
        if os.path.exists(local_history_path):
            shutil.copy2(local_history_path, tuner_history_path)
        # Run cody_tuner.exe ask
        run_step([CODY_TUNER_EXE, "ask"], "Requesting new parameters", cwd=CODYTUNER_DIR)
        # Copy updated params.json and tuner_history.json back
        shutil.copy2(tuner_params_path, local_params_path)
        if os.path.exists(tuner_history_path):
            shutil.copy2(tuner_history_path, local_history_path)

        # B. Apply those parameters to the Rust source code
        run_step(["python", "edit_consts.py", PARAMS_FILE], "Patching engine_consts.rs")

        # C. Rebuild the engine (this creates a new cody.exe)
        run_step(["python", "build_engine.py"], "Building candidate engine")

        # D. Test Candidate (ENGINE_SRC_BIN) vs. Static Baseline (BASELINE_BIN)
        # Arguments: <candidate> <games> <results_json> <baseline>
        selfplay_output = run_step([
            "python", "selfplay.py",
            ENGINE_SRC_BIN,
            str(GAMES_PER_MATCH),
            RESULTS_FILE,
            BASELINE_BIN
        ], "Running match (Candidate vs. Static Baseline)")

        # Extract Elo difference from CuteChess output
        import re
        elo_diff = 0.0
        if selfplay_output:
            match = re.search(r"Elo difference:\s*([\-\d\.]+)", selfplay_output)
            if match:
                try:
                    elo_diff = float(match.group(1))
                except ValueError:
                    print(f"WARNING: Invalid Elo difference value '{match.group(1)}', setting elo_diff = 0.0")
                    elo_diff = 0.0
        # Optional: Print summary of the round
        try:
            with open(os.path.join(TUNING_DIR, RESULTS_FILE), 'r') as f:
                res = json.load(f)
                print(f"Round {i+1} Results: {res.get('wins')}W | {res.get('losses')}L | {res.get('draws')}D")
                # Record the result with cody_tuner.exe
                trial_id = None
                # Read trial_id from local tuner_history.json if available
                if os.path.exists(local_history_path):
                    try:
                        with open(local_history_path, 'r') as th:
                            history = json.load(th)
                            if isinstance(history, list) and history:
                                trial_id = history[-1].get("id")
                            elif isinstance(history, dict):
                                trial_id = history.get("id")
                    except Exception:
                        pass
                if trial_id is not None:
                    score = elo_diff
                    # Sync files to cody_tuner.exe directory
                    shutil.copy2(local_params_path, tuner_params_path)
                    if os.path.exists(local_history_path):
                        shutil.copy2(local_history_path, tuner_history_path)
                    run_step([
                        CODY_TUNER_EXE,
                        "tell",
                        str(trial_id),
                        str(score)
                    ], f"Recording score {score} for trial {trial_id}", cwd=CODYTUNER_DIR)
                    # Sync files back
                    shutil.copy2(tuner_params_path, local_params_path)
                    if os.path.exists(tuner_history_path):
                        shutil.copy2(tuner_history_path, local_history_path)
                else:
                    print("WARNING: trial_id not found for result recording.")
        except Exception:
            pass

if __name__ == "__main__":
    main()