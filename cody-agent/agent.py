import os
import sys
import json
import subprocess
import datetime
import requests
from pathlib import Path
from openai import OpenAI

# -----------------------------
# 1. Configuration & Secrets
# -----------------------------
# ... inside the config loading section ...
AGENT_DIR = Path(__file__).resolve().parent
CONFIG_PATH = AGENT_DIR / "config.json"
SAMPLE_PATH = AGENT_DIR / "config.sample.json"

if not CONFIG_PATH.exists():
    if SAMPLE_PATH.exists():
        print("❌ Error: 'config.json' not found.")
        print(f"👉 Please copy '{SAMPLE_PATH.name}' to 'config.json' and edit your settings.")
    else:
        print("❌ Error: No configuration files found.")
    sys.exit(1)

CONFIG = json.load(open(CONFIG_PATH))

OPENAI_KEY = os.environ.get("OPENAI_API_KEY")
GITHUB_TOKEN = os.environ.get("GITHUB_TOKEN")
GITHUB_REPO = CONFIG.get("github_repo", "yourusername/cody") 

if not CONFIG.get("use_local") and not OPENAI_KEY:
    print(f"\n❌ Error: OPENAI_API_KEY environment variable not set")
    print(f"\n   Set your API key:")
    print(f"   export OPENAI_API_KEY=sk-...")
    print(f"\n   Or configure 'use_local': true in config.json to use a local LLM.\n")
    sys.exit(1)
if not GITHUB_TOKEN:
    print(f"\n❌ Error: GITHUB_TOKEN environment variable not set")
    print(f"\n   Set your GitHub token:")
    print(f"   export GITHUB_TOKEN=ghp_...\n")
    sys.exit(1)

BRANCH_PREFIX = CONFIG["branch_prefix"]
MODEL = CONFIG["model"]

# Load System Prompt
REPO_ROOT = AGENT_DIR.parent
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
        if not OPENAI_KEY:
            print(f"\n❌ Error: OPENAI_API_KEY environment variable not set")
            print(f"\n   Set your API key:")
            print(f"   export OPENAI_API_KEY=sk-...")
            print(f"\n   Or configure 'use_local': true in config.json to use a local LLM.\n")
            sys.exit(1)
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
    """Saves the patch and attempts to apply it with better diagnostics."""
    timestamp = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%d-%H%M%S")
    temp_patch = repo_path / f"ai_improvement_{timestamp}.patch"
    
    # Save the patch first so it's available for manual inspection
    temp_patch.write_text(patch_content)
    print(f"📄 Patch saved to: {temp_patch}")

    # Create a local branch BEFORE applying so the failure is contained
    debug_branch = f"debug-ai-{timestamp}"
    run(f"git checkout -b {debug_branch}", cwd=repo_path)

    # Use --verbose to get detailed error messages from git
    print(f"Attempting to apply patch to branch {debug_branch}...")
    stdout, ok = run(f"git apply --verbose {temp_patch.name}", cwd=repo_path)
    
    if not ok:
        print(f"❌ PATCH FAILED. Diagnostics:")
        print(f"   1. Detailed log: {stdout}")
        print(f"   2. You can inspect the failure in branch: {debug_branch}")
        print(f"   3. The raw patch is available at: {temp_patch.name}")
        # We DO NOT delete the patch or switch branches so you can debug
        return False

    # If it worked, we can delete the temp file but keep the branch
    temp_patch.unlink()
    return True

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