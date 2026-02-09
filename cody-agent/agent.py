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

# ... [Keep your run, get_repo_root, and ensure_clean_repo functions as they are] ...

# -----------------------------
# Step 4: Commit, push, PR (Fixed GITHUB_REPO usage)
# -----------------------------
def create_pr(repo_path):
    timestamp = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%d-%H%M%S")
    branch = f"{BRANCH_PREFIX}{timestamp}"

    run(f"git checkout -b {branch}", cwd=repo_path)
    run("git add .", cwd=repo_path)
    run(f'git commit -m "AI improvement {timestamp}"', cwd=repo_path)
    run(f"git push origin {branch}", cwd=repo_path)

    url = f"https://api.github.com/repos/{GITHUB_REPO}/pulls"
    headers = {
        "Authorization": f"token {GITHUB_TOKEN}",
        "Accept": "application/vnd.github+json"
    }
    data = {
        "title": f"AI Improvement {timestamp}",
        "head": branch,
        "base": "main",
        "body": "Automated AI-generated improvement using local LLM."
    }

    r = requests.post(url, headers=headers, json=data)
    if r.status_code >= 300:
        raise RuntimeError(f"PR creation failed: {r.text}")

    print("PR created:", r.json()["html_url"])

# ... [Keep main function as it is] ...
