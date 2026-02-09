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
GITHUB_REPO = CONFIG.get("github_repo", "yourusername/cody") 

if not CONFIG.get("use_local") and not OPENAI_KEY:
    raise RuntimeError("Missing OPENAI_API_KEY for non-local mode")

if not GITHUB_TOKEN:
    raise RuntimeError("Missing GITHUB_TOKEN environment variable")

BRANCH_PREFIX = CONFIG["branch_prefix"]
MODEL = CONFIG["model"]

SYSTEM_PROMPT_PATH = Path(__file__).resolve().parents[1] / ".github" / "ai" / "system.md"
SYSTEM_PROMPT = SYSTEM_PROMPT_PATH.read_text() if SYSTEM_PROMPT_PATH.exists() else "You are a helpful coding assistant."

# -----------------------------
# Updated Run Helper: Returns Success
# -----------------------------
def run(command, cwd=None):
    """Executes a command and returns (stdout, success_bool)."""
    print(f"Executing: {command}")
    result = subprocess.run(
        command, shell=True, capture_output=True, text=True, cwd=cwd, check=False
    )
    
    if result.returncode != 0:
        print(f"❌ FAILED: {command}")
        if result.stderr:
            print(f"Error Output: {result.stderr.strip()}")
        return result.stdout.strip(), False
    
    print(f"✅ SUCCESS: {command}")
    return result.stdout.strip(), True

# -----------------------------
# New: Local Validation Loop
# -----------------------------
def validate_engine(repo_path):
    """Runs the required Cargo checks. All must pass for validation to succeed."""
    print("\n--- Starting Quality Assurance Checks ---")
    
    # 1. Format code
    _, ok = run("cargo fmt", cwd=repo_path)
    if not ok: return False
    
    # 2. Build for release
    print("Building engine (release)...")
    _, ok = run("cargo build --release", cwd=repo_path)
    if not ok: return False
    
    # 3. Run unit tests
    print("Running cargo tests...")
    _, ok = run("cargo test", cwd=repo_path)
    if not ok: return False
    
    # 4. Performance/Correctness check (Perft 5)
    print("Running Perft 5 verification...")
    _, ok = run("cargo run --release -p engine -- perft 5", cwd=repo_path)
    if not ok: return False
    
    print("--- All Checks Passed! Proceeding to PR ---\n")
    return True

# -----------------------------
# Step 4: Commit with Validation
# -----------------------------
def create_pr(repo_path):
    # Only proceed if the code passes your requirements
    if not validate_engine(repo_path):
        print("🛑 STOPPING: AI-generated code failed validation. No changes will be pushed.")
        return False

    timestamp = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%d-%H%M%S")
    branch = f"{BRANCH_PREFIX}{timestamp}"

    # Push changes
    run(f"git checkout -b {branch}", cwd=repo_path)
    run("git add .", cwd=repo_path)
    run(f'git commit -m "Verified AI improvement {timestamp}"', cwd=repo_path)
    run(f"git push origin {branch}", cwd=repo_path)

    # PR Logic
    url = f"https://api.github.com/repos/{GITHUB_REPO}/pulls"
    headers = {"Authorization": f"token {GITHUB_TOKEN}", "Accept": "application/vnd.github+json"}
    data = {
        "title": f"Verified AI Improvement {timestamp}",
        "head": branch,
        "base": "main",
        "body": "Automated improvement that passed cargo fmt, build, test, and perft 5."
    }

    r = requests.post(url, headers=headers, json=data)
    if r.status_code >= 300:
        print(f"⚠️ PR failed: {r.text}")
    else:
        print("🚀 PR created successfully:", r.json()["html_url"])
    return True

# ... (main loop calls create_pr)