import os
import json
import subprocess
import tempfile
import datetime
import requests

from pathlib import Path

# -----------------------------
# Load config
# -----------------------------
AGENT_DIR = Path(__file__).resolve().parent
CONFIG_PATH = AGENT_DIR / "config.json"
CONFIG = json.load(open(CONFIG_PATH))
GITHUB_REPO = CONFIG["github_repo"]
GITHUB_TOKEN = CONFIG["github_token"]
OPENAI_KEY = CONFIG["openai_api_key"]
BRANCH_PREFIX = CONFIG["branch_prefix"]
MODEL = CONFIG["model"]


# Path to the system prompt inside the Cody repo
SYSTEM_PROMPT_PATH = Path(__file__).resolve().parents[1] / ".github" / "ai" / "system.md"
SYSTEM_PROMPT = SYSTEM_PROMPT_PATH.read_text()


# -----------------------------
# Helper: call OpenAI
# -----------------------------
from openai import OpenAI

def call_ai(prompt):
    client = OpenAI(api_key=OPENAI_KEY)

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
# Helper: run shell commands
# -----------------------------
def run(cmd, cwd=None):
    print(f"Running: {cmd}")
    result = subprocess.run(cmd, shell=True, cwd=cwd,
                            stdout=subprocess.PIPE,
                            stderr=subprocess.PIPE,
                            text=True)
    if result.returncode != 0:
        print(result.stdout)
        print(result.stderr)
        raise RuntimeError(f"Command failed: {cmd}")
    return result.stdout


def get_repo_root():
    # agent.py is in Cody/cody-agent/
    # repo root is one directory up
    return Path(__file__).resolve().parents[1]


def ensure_clean_repo(repo_path):
    status = run("git status --porcelain", cwd=repo_path)
    if status.strip():
        raise RuntimeError("Repo has uncommitted changes. Agent will not proceed.")


# -----------------------------
# Step 2: Ask AI for a patch
# -----------------------------
def generate_patch(repo_path):
    # Read all Rust files
    code = ""
    for path in repo_path.rglob("*.rs"):
        rel = path.relative_to(repo_path)
        code += f"\n\n// FILE: {rel}\n"
        code += path.read_text()

    prompt = f"""
You are Cody's autonomous improvement agent.

Here is the current codebase:

{code}

Generate a unified diff patch (git format) that improves the engine.
Follow these rules:
- Only output the patch, nothing else.
- The patch must apply cleanly with `git apply`.
- Keep changes focused and incremental.
- Follow idiomatic Rust.
- Use best-practice crates when appropriate.
- Maintain correctness.
- Do not break build or tests.
"""

    return call_ai(prompt)


# -----------------------------
# Step 3: Apply patch
# -----------------------------
def apply_patch(repo_path, patch_text):
    patch_file = repo_path / "patch.diff"
    patch_file.write_text(patch_text)

    try:
        run(f"git apply patch.diff", cwd=repo_path)
    except Exception:
        print("Patch failed to apply.")
        return False

    return True


# -----------------------------
# Step 4: Commit, push, PR
# -----------------------------
def create_pr(repo_path):
    timestamp = datetime.datetime.utcnow().strftime("%Y%m%d-%H%M%S")
    branch = f"{BRANCH_PREFIX}{timestamp}"

    run(f"git checkout -b {branch}", cwd=repo_path)
    run("git add .", cwd=repo_path)
    run(f'git commit -m "AI-generated improvement {timestamp}"', cwd=repo_path)
    run(f"git push origin {branch}", cwd=repo_path)

    # Create PR via GitHub API
    url = f"https://api.github.com/repos/{GITHUB_REPO}/pulls"
    headers = {
        "Authorization": f"token {GITHUB_TOKEN}",
        "Accept": "application/vnd.github+json"
    }
    data = {
        "title": f"AI Improvement {timestamp}",
        "head": branch,
        "base": "main",
        "body": "Automated AI-generated improvement."
    }

    r = requests.post(url, headers=headers, json=data)
    if r.status_code >= 300:
        raise RuntimeError(f"PR creation failed: {r.text}")

    print("PR created:", r.json()["html_url"])


# -----------------------------
# Main
# -----------------------------
def main():
    repo = get_repo_root()
    ensure_clean_repo(repo)
    patch = generate_patch(repo)

    if not apply_patch(repo, patch):
        print("Skipping PR due to patch failure.")
        return

    create_pr(repo)


if __name__ == "__main__":
    main()
