import os
import json
import subprocess
import datetime
import requests
from pathlib import Path
from openai import OpenAI

# -----------------------------
# 1. Configuration & Secrets
# -----------------------------
AGENT_DIR = Path(__file__).resolve().parent
# Assuming the script is in /cody-agent/ and the project is in the parent dir
REPO_ROOT = AGENT_DIR.parent 
CONFIG_PATH = AGENT_DIR / "config.json"

if not CONFIG_PATH.exists():
    raise FileNotFoundError(f"Config file not found at {CONFIG_PATH}")

CONFIG = json.load(open(CONFIG_PATH))

OPENAI_KEY = os.environ.get("OPENAI_API_KEY")
GITHUB_TOKEN = os.environ.get("GITHUB_TOKEN")
GITHUB_REPO = CONFIG.get("github_repo", "yourusername/cody") 

if not CONFIG.get("use_local") and not OPENAI_KEY:
    raise RuntimeError("Missing OPENAI_API_KEY for cloud mode.")
if not GITHUB_TOKEN:
    raise RuntimeError("Missing GITHUB_TOKEN environment variable.")

BRANCH_PREFIX = CONFIG["branch_prefix"]
MODEL = CONFIG["model"]

# Load System Prompt
SYSTEM_PROMPT_PATH = REPO_ROOT / ".github" / "ai" / "system.md"
SYSTEM_PROMPT = SYSTEM_PROMPT_PATH.read_text() if SYSTEM_PROMPT_PATH.exists() else "You are a senior Rust engineer building a chess engine."

# -----------------------------
# 2. Helper Functions
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

def call_ai(prompt):
    """Sends the context to the LLM (Local or OpenAI)."""
    if CONFIG.get("use_local"):
        client = OpenAI(api_key="ollama", base_url=CONFIG.get("api_base", "http://localhost:11434/v1"))
    else:
        client = OpenAI(api_key=OPENAI_KEY)

    print(f"🤖 AI is thinking (Model: {MODEL})...")
    response = client.chat.completions.create(
        model=MODEL,
        messages=[
            {"role": "system", "content": SYSTEM_PROMPT},
            {"role": "user", "content": prompt}
        ],
        temperature=0.4
    )
    return response.choices[0].message.content

# -----------------------------
# 3. Quality Assurance (Your Requirements)
# -----------------------------
def validate_engine(repo_path):
    """Runs the required Cargo checks in order."""
    print("\n🛡️ Starting Quality Assurance Checks...")
    
    # 1. Format code first
    print("Formatting code...")
    _, ok = run("cargo fmt", cwd=repo_path)
    if not ok: return False
    
    # 2. Build for release
    print("Building engine (release mode)...")
    _, ok = run("cargo build --release", cwd=repo_path)
    if not ok: return False
    
    # 3. Run unit tests
    print("Running unit tests...")
    _, ok = run("cargo test", cwd=repo_path)
    if not ok: return False
    
    # 4. Perft 5 verification
    print("Running Performance Test (Perft 5)...")
    _, ok = run("cargo run --release -p engine -- perft 5", cwd=repo_path)
    if not ok: return False
    
    print("✨ QA Passed: All verification steps succeeded.\n")
    return True

# -----------------------------
# 4. The Improvement Workflow
# -----------------------------
def generate_patch(repo_path):
    """Gathers all Rust code and asks AI for an improvement."""
    code_context = ""
    for path in repo_path.rglob("*.rs"):
        if "target" in str(path): continue
        rel_path = path.relative_to(repo_path)
        code_context += f"\n\n// FILE: {rel_path}\n{path.read_text()}"

    prompt = f"Review the following Rust chess engine code and provide a single 'unified diff' patch to improve its search or evaluation logic:\n\n{code_context}"
    return call_ai(prompt)

def apply_patch(repo_path, patch_content):
    """Applies the AI's patch to the local files."""
    patch_file = repo_path / "improvement.patch"
    patch_file.write_text(patch_content)
    _, ok = run(f"git apply improvement.patch", cwd=repo_path)
    patch_file.unlink() # Cleanup
    return ok

def create_pr(repo_path):
    """Validates code locally, then pushes to GitHub and opens a PR."""
    if not validate_engine(repo_path):
        print("🛑 STOPPING: Code failed local validation. No changes will be pushed.")
        return False

    timestamp = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%d-%H%M%S")
    branch = f"{BRANCH_PREFIX}{timestamp}"

    # Push to GitHub
    run(f"git checkout -b {branch}", cwd=repo_path)
    run("git add .", cwd=repo_path)
    run(f'git commit -m "Verified AI improvement {timestamp}"', cwd=repo_path)
    run(f"git push origin {branch}", cwd=repo_path)

    # API Request for PR
    url = f"https://api.github.com/repos/{GITHUB_REPO}/pulls"
    headers = {"Authorization": f"token {GITHUB_TOKEN}", "Accept": "application/vnd.github+json"}
    data = {
        "title": f"Verified AI Improvement {timestamp}",
        "head": branch,
        "base": "main",
        "body": "This automated improvement passed: cargo fmt, release build, tests, and perft 5."
    }

    r = requests.post(url, headers=headers, json=data)
    if r.status_code >= 300:
        print(f"⚠️ PR creation failed: {r.text}")
    else:
        print(f"🚀 SUCCESS: PR created at {r.json()['html_url']}")
    return True

# -----------------------------
# 5. The Main Entry Point
# -----------------------------
def main():
    print(f"--- Cody AI Agent Started at {datetime.datetime.now()} ---")
    
    # Ensure we are in a clean state
    run("git checkout main", cwd=REPO_ROOT)
    
    # 1. Ask AI for a change
    patch = generate_patch(REPO_ROOT)
    
    # 2. Apply it
    if apply_patch(REPO_ROOT, patch):
        # 3. Validate and PR
        create_pr(REPO_ROOT)
    else:
        print("❌ AI generated an invalid patch that could not be applied.")

if __name__ == "__main__":
    main()