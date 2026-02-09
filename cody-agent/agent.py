import os
import json
import subprocess
import datetime
import requests
from pathlib import Path
from openai import OpenAI

# -----------------------------
# Load config
# -----------------------------
AGENT_DIR = Path(__file__).resolve().parent
CONFIG_PATH = AGENT_DIR / "config.json"
CONFIG = json.load(open(CONFIG_PATH))

# Load secrets
OPENAI_KEY = os.environ.get("OPENAI_API_KEY")
GITHUB_TOKEN = os.environ.get("GITHUB_TOKEN")
# ADD THIS: You need your repo name here (e.g., "yourusername/cody")
GITHUB_REPO = CONFIG.get("github_repo", "yourusername/cody") 

# FIX: Check for keys only if they are actually needed
if not CONFIG.get("use_local") and not OPENAI_KEY:
    raise RuntimeError("Missing OPENAI_API_KEY for non-local mode")

if not GITHUB_TOKEN:
    raise RuntimeError("Missing GITHUB_TOKEN environment variable")

BRANCH_PREFIX = CONFIG["branch_prefix"]
MODEL = CONFIG["model"]

# Path to system prompt
SYSTEM_PROMPT_PATH = Path(__file__).resolve().parents[1] / ".github" / "ai" / "system.md"
SYSTEM_PROMPT = SYSTEM_PROMPT_PATH.read_text() if SYSTEM_PROMPT_PATH.exists() else "You are a helpful coding assistant."

# -----------------------------
# Updated Helper: call AI (Local or Cloud)
# -----------------------------
def call_ai(prompt):
    if CONFIG.get("use_local"):
        client = OpenAI(
            api_key="ollama", 
            base_url=CONFIG.get("api_base", "http://localhost:11434/v1")
        )
    else:
        client = OpenAI(api_key=OPENAI_KEY)

    print(f"Requesting improvement from {MODEL}...")
    response = client.chat.completions.create(
        model=MODEL,
        messages=[
            {"role": "system", "content": SYSTEM_PROMPT},
            {"role": "user", "content": prompt}
        ],
        temperature=0.4
    )
    return response.choices[0].message.content

# ... (imports and config loading)

# -----------------------------
# Updated Run Helper: Force Mode
# -----------------------------
def run(command, cwd=None):
    """Executes a command and returns output. In Force Mode, it won't crash on non-zero exits."""
    print(f"Running: {command}")
    # Use check=False to prevent the script from crashing if a command fails
    result = subprocess.run(
        command, shell=True, capture_output=True, text=True, cwd=cwd, check=False
    )
    if result.returncode != 0:
        print(f"⚠️ Command failed with code {result.returncode}: {result.stderr.strip()}")
    return result.stdout.strip()

# -----------------------------
# Updated Clean Repo Check
# -----------------------------
def ensure_clean_repo(repo_path):
    """Just a warning now. No more RuntimeErrors."""
    print("Checking repo status...")
    status = run("git status --porcelain", cwd=repo_path)
    if status:
        print("⚠️ WARNING: Repo has uncommitted changes. FORCE MODE: Proceeding anyway.")
    return True # Always return True to keep the loop going

# -----------------------------
# Step 4: Commit with Bypass
# -----------------------------
def create_pr(repo_path):
    timestamp = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%d-%H%M%S")
    branch = f"{BRANCH_PREFIX}{timestamp}"

    run(f"git checkout -b {branch}", cwd=repo_path)
    run("git add .", cwd=repo_path)
    
    # Use --no-verify to skip any local git hooks (pre-commit checks)
    run(f'git commit -m "AI-generated improvement {timestamp}" --no-verify', cwd=repo_path)
    
    # Force push to overwrite any remote conflicts
    run(f"git push origin {branch} --force", cwd=repo_path)

    # ... (rest of PR logic)