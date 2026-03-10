"""
Master orchestration script for automated tuning.
Runs parameter editing, build, self-play, and optimization in sequence.
Usage: python tune.py
"""
import subprocess
import json
import os

TUNING_DIR = os.path.dirname(os.path.abspath(__file__))
PARAMS_FILE = os.path.join(TUNING_DIR, 'params.json')
RESULTS_FILE = os.path.join(TUNING_DIR, 'results.json')
ENGINE_PATH = os.path.abspath(os.path.join(TUNING_DIR, '../target/release/engine.exe'))

def regenerate_params():
    subprocess.run(['python', 'edit_consts.py', '--extract'], cwd=TUNING_DIR, check=True)

# Regenerate params.json ONCE at the start
regenerate_params()

max_iters = 5  # Number of tuning rounds
for i in range(max_iters):
    print(f'--- Tuning round {i+1} ---')
    # Step 1: Edit constants (apply params.json to engine_consts.rs)
    subprocess.run(['python', 'edit_consts.py', PARAMS_FILE], cwd=TUNING_DIR, check=True)
    # Step 2: Build engine
    subprocess.run(['python', 'build_engine.py'], cwd=TUNING_DIR, check=True)
    # Step 3: Self-play
    subprocess.run(['python', 'selfplay.py', ENGINE_PATH, '10', RESULTS_FILE], cwd=TUNING_DIR, check=True)
    # Step 4: Bayesian optimization (update params.json)
    subprocess.run(['python', 'bayes_opt.py', RESULTS_FILE, PARAMS_FILE], cwd=TUNING_DIR, check=True)
    # Step 5: Apply new params to engine_consts.rs
    subprocess.run(['python', 'edit_consts.py', PARAMS_FILE], cwd=TUNING_DIR, check=True)
print('Tuning complete. Final params:')
with open(PARAMS_FILE, 'r', encoding='utf-8') as f:
    print(json.load(f))
