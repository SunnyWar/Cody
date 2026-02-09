"""
Features Executor Agent

Implements a world-class chess engine feature from the TODO list.
"""

import os
import sys
import json
import subprocess
from pathlib import Path
from openai import OpenAI
from todo_manager import TodoList


def load_config():
    """Load configuration."""
    config_path = Path(__file__).parent / "config.json"
    
    if not config_path.exists():
        print(f"❌ Error: Configuration file not found at {config_path}")
        print(f"\n   Please create config.json by copying config.sample.json:")
        print(f"   cp {Path(__file__).parent / 'config.sample.json'} {config_path}")
        sys.exit(1)
    
    try:
        with open(config_path) as f:
            config = json.load(f)
        if not config:
            raise ValueError("Config file is empty")
        return config
    except json.JSONDecodeError as e:
        print(f"❌ Error: Invalid JSON in {config_path}")
        print(f"   {e}")
        sys.exit(1)
    except Exception as e:
        print(f"❌ Error reading config file: {e}")
        sys.exit(1)


def get_prompt_template():
    """Load the features execution prompt."""
    repo_root = Path(__file__).parent.parent
    prompt_path = repo_root / ".github" / "ai" / "prompts" / "features_execution.md"
    return prompt_path.read_text(encoding='utf-8')


def gather_relevant_code(repo_root: Path, files: list) -> str:
    """Gather code from specific files."""
    code_context = []
    
    for file_path in files:
        full_path = repo_root / file_path
        if full_path.exists():
            content = full_path.read_text(encoding='utf-8')
            code_context.append(f"\n// ========== FILE: {file_path} ==========\n{content}")
        else:
            print(f"⚠️ File not found: {file_path}")
    
    return "\n".join(code_context)


def gather_architecture_context(repo_root: Path) -> str:
    """Gather architecture and design docs."""
    context = []
    
    for doc in ["architecture.md", ".github/copilot-instructions.md"]:
        doc_path = repo_root / doc
        if doc_path.exists():
            context.append(f"\n// ========== {doc} ==========\n{doc_path.read_text(encoding='utf-8')}")
    
    return "\n".join(context)


def call_ai(prompt: str, config: dict) -> str:
    """Call the AI with the prompt."""
    if config.get("use_local"):
        client = OpenAI(
            api_key="ollama", 
            base_url=config.get("api_base", "http://localhost:11434/v1")
        )
    else:
        api_key = os.environ.get("OPENAI_API_KEY")
        if not api_key:
            print(f"\n❌ Error: OPENAI_API_KEY environment variable not set")
            print(f"\n   Set your API key:")
            print(f"   export OPENAI_API_KEY=sk-...")
            print(f"\n   Or configure 'use_local': true in config.json to use a local LLM.\n")
            sys.exit(1)
        client = OpenAI(api_key=api_key)
    
    model = config["model"]
    print(f"🤖 Implementing feature with {model}...")
    
    response = client.chat.completions.create(
        model=model,
        messages=[
            {"role": "system", "content": "You are a chess engine expert implementing world-class features."},
            {"role": "user", "content": prompt}
        ],
        temperature=0.3
    )
    
    return response.choices[0].message.content


def extract_patch(response: str) -> str:
    """Extract the patch from AI response."""
    if "```diff" in response:
        start = response.find("```diff") + 7
        end = response.find("```", start)
        return response[start:end].strip()
    elif "diff --git" in response:
        start = response.find("diff --git")
        return response[start:].strip()
    else:
        print("⚠️ No clear diff block found in response")
        return response


def apply_patch(repo_root: Path, patch: str) -> bool:
    """Apply the patch to the repository."""
    patch_file = repo_root / "temp_feature.patch"
    
    try:
        patch_file.write_text(patch)
        print(f"📄 Patch saved to: {patch_file}")
        
        result = subprocess.run(
            ["git", "apply", "--verbose", str(patch_file)],
            cwd=repo_root,
            capture_output=True,
            text=True
        )
        
        if result.returncode == 0:
            print("✅ Patch applied successfully")
            patch_file.unlink()
            return True
        else:
            print(f"❌ Failed to apply patch:")
            print(result.stderr)
            print(f"Patch saved at {patch_file} for manual review")
            return False
            
    except Exception as e:
        print(f"❌ Error applying patch: {e}")
        return False


def validate_changes(repo_root: Path) -> bool:
    """Run comprehensive validation."""
    print("\n🛡️ Validating feature implementation...")
    
    steps = [
        ("Format", ["cargo", "fmt"]),
        ("Build (debug)", ["cargo", "build"]),
        ("Build (release)", ["cargo", "build", "--release"]),
        ("Tests", ["cargo", "test"]),
        ("Perft verification", ["cargo", "run", "--release", "-p", "engine", "--", "perft", "5"]),
    ]
    
    for step_name, command in steps:
        print(f"\n  Running: {step_name}...")
        result = subprocess.run(
            command,
            cwd=repo_root,
            capture_output=True,
            text=True,
            timeout=600
        )
        
        if result.returncode != 0:
            print(f"  ❌ {step_name} failed:")
            print(result.stderr)
            return False
        else:
            print(f"  ✅ {step_name} passed")
    
    return True


def calculate_diff_size(repo_root: Path) -> str:
    """Calculate the size of changes."""
    result = subprocess.run(
        ["git", "diff", "--stat"],
        cwd=repo_root,
        capture_output=True,
        text=True
    )
    
    return result.stdout.strip()


def execute_feature(item_id: str, repo_root: Path, config: dict) -> tuple[bool, str]:
    """Execute a feature implementation. Returns (success, diff_size)."""
    print("=" * 60)
    print(f"IMPLEMENTING FEATURE: {item_id}")
    print("=" * 60)
    
    # Load TODO list
    todo_list = TodoList("features", repo_root)
    
    item = None
    for todo_item in todo_list.items:
        if todo_item.id == item_id:
            item = todo_item
            break
    
    if not item:
        print(f"❌ Item {item_id} not found in TODO list")
        return False, "none"
    
    if item.status == "completed":
        print(f"⏭️ Item {item_id} is already completed")
        return True, "none"
    
    # Mark as in progress
    todo_list.mark_in_progress(item_id)
    todo_list.save()
    
    # Build the prompt
    prompt_template = get_prompt_template()
    
    feature_details = f"""
ID: {item.id}
Title: {item.title}
Priority: {item.priority}
Category: {item.category}
ELO Impact: {item.metadata.get('elo_impact', 'Unknown')}
Complexity: {item.estimated_complexity}
Dependencies: {', '.join(item.dependencies) if item.dependencies else 'None'}

Description: {item.description}

Implementation Approach: {item.metadata.get('implementation_approach', 'See description')}

References: {item.metadata.get('references', 'None')}
"""
    
    # Gather context
    arch_context = gather_architecture_context(repo_root)
    
    code_context = ""
    if item.files_affected:
        code_context = gather_relevant_code(repo_root, item.files_affected)
    else:
        # Gather search and engine code by default
        default_files = [
            "engine/src/search/engine.rs",
            "engine/src/search/search.rs",
            "engine/src/api/uciapi.rs",
            "bitboard/src/position.rs",
        ]
        code_context = gather_relevant_code(repo_root, default_files)
    
    full_prompt = prompt_template.replace("{FEATURE_DETAILS}", feature_details)
    full_prompt += f"\n\n## ARCHITECTURE CONTEXT\n\n{arch_context}"
    full_prompt += f"\n\n## CODE CONTEXT\n\n{code_context}"
    
    # Call AI
    response = call_ai(full_prompt, config)
    
    # Extract and apply patch
    patch = extract_patch(response)
    
    if not patch:
        print("❌ Failed to extract patch from response")
        return False, "none"
    
    if not apply_patch(repo_root, patch):
        print("❌ Failed to apply patch")
        return False, "none"
    
    # Calculate diff size
    diff_size = calculate_diff_size(repo_root)
    print(f"\n📏 Changes:\n{diff_size}")
    
    # Determine if diff is large
    lines_changed = sum(
        int(line.split()[0]) 
        for line in diff_size.split('\n') 
        if line and line[0].isdigit()
    ) if diff_size else 0
    
    size_category = "large" if lines_changed > 100 else "small"
    
    # Validate
    if not validate_changes(repo_root):
        print("❌ Validation failed, rolling back...")
        subprocess.run(["git", "checkout", "."], cwd=repo_root)
        return False, size_category
    
    # Mark as completed
    todo_list.mark_completed(item_id)
    todo_list.save()
    
    print(f"\n✅ Feature {item_id} implemented successfully")
    return True, size_category


def main():
    """Main entry point."""
    import sys
    
    if len(sys.argv) < 2:
        print("Usage: python features_executor.py <item_id>")
        print("   or: python features_executor.py next")
        sys.exit(1)
    
    config = load_config()
    repo_root = Path(__file__).parent.parent
    
    item_id = sys.argv[1]
    
    if item_id == "next":
        todo_list = TodoList("features", repo_root)
        next_item = todo_list.get_next_item()
        if not next_item:
            print("No items available to work on")
            sys.exit(0)
        item_id = next_item.id
        print(f"Working on next item: {item_id}")
    
    success, diff_size = execute_feature(item_id, repo_root, config)
    
    if success:
        print(f"\n{'=' * 60}")
        print(f"Feature completed successfully (diff size: {diff_size})")
        print(f"{'=' * 60}")
    else:
        print(f"\n{'=' * 60}")
        print("Feature implementation failed")
        print(f"{'=' * 60}")
        sys.exit(1)


if __name__ == "__main__":
    main()
