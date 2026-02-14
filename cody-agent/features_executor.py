"""
Features Executor Agent

Implements a world-class chess engine feature from the TODO list.
"""

import sys
import json
import subprocess
from datetime import datetime
from pathlib import Path
from todo_manager import TodoList
from executor_state import record_last_change
from validation import ensure_builds_or_fix, rollback_changes
from agent_runner import run_agent


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


def call_ai(prompt: str, config: dict, repo_root: Path) -> str:
    """Call the agent with the prompt."""
    model = config.get("model")
    if model:
        print(f"🤖 Implementing feature with {model}...")
    else:
        print("🤖 Implementing feature...")

    system_prompt = "You are a chess engine expert implementing world-class features."

    return run_agent(system_prompt, prompt, config, repo_root, "features_executor")


def extract_file_content(response: str) -> tuple[str, str]:
    """Extract file path and content from LLM response.
    Returns (file_path, content) or (None, None) if extraction fails.
    """
    # Look for file path in comment at top of code block
    if "```rust" in response:
        start = response.find("```rust") + 7
        end = response.find("```", start)
        if end != -1:
            code = response[start:end].strip()
            # Extract file path from comment
            lines = code.split("\n")
            file_path = None
            for i, line in enumerate(lines[:5]):  # Check first 5 lines
                if "//" in line and (".rs" in line or "/" in line):
                    # Extract path from comment
                    path_part = line.split("//", 1)[1].strip()
                    if path_part and not path_part.startswith(" "):
                        file_path = path_part
                    break
            
            # Remove comment lines from code
            code_lines = []
            skip_comments = True
            for line in lines:
                if skip_comments and line.strip().startswith("//"):
                    continue
                skip_comments = False
                code_lines.append(line)
            
            content = "\n".join(code_lines).strip()
            return file_path, content
    
    print("⚠️ No clear code block found in response")
    return None, None


def apply_code_changes(repo_root: Path, file_path: str, new_content: str) -> bool:
    """Write new content directly to file."""
    try:
        full_path = repo_root / file_path
        if not full_path.parent.exists():
            print(f"❌ Parent directory does not exist: {full_path.parent}")
            return False
        
        # Backup original
        backup_content = None
        if full_path.exists():
            backup_content = full_path.read_text(encoding="utf-8")
        
        # Write new content
        full_path.write_text(new_content, encoding="utf-8")
        print(f"✅ Updated {file_path}")
        return True
        
    except Exception as e:
        print(f"❌ Error writing file: {e}")
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
    
    # MANDATORY PRE-VALIDATION: Ensure project builds BEFORE making changes
    if not ensure_builds_or_fix(repo_root, config, "PRE-CHANGE"):
        print("❌ CRITICAL: Project does not build before changes and could not be fixed.")
        print("   Aborting to prevent further damage.")
        todo_list.mark_failed(item_id)
        todo_list.save()
        return False, "none"
    
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
    
    # Call AI with error handling
    try:
        response = call_ai(full_prompt, config, repo_root)
    except Exception as e:
        logs_dir = repo_root / ".orchestrator_logs"
        logs_dir.mkdir(parents=True, exist_ok=True)
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        error_path = logs_dir / f"features_exec_ai_error_{timestamp}.txt"
        with error_path.open("w", encoding="utf-8", errors="replace") as f:
            f.write(f"AI API Error: {type(e).__name__}\n")
            f.write(f"Error message: {str(e)}\n")
            f.write(f"\nitem_id: {item_id}\n")
        print(f"❌ AI API Error: {type(e).__name__}: {str(e)[:200]}")
        print(f"📄 Error details saved to: {error_path}")
        return False, "error"
    
    # Extract and apply changes
    file_path, new_content = extract_file_content(response)
    
    if not file_path or not new_content:
        print("❌ Failed to extract file content from response")
        # Save response for debugging
        debug_file = repo_root / "temp_feature_response.txt"
        debug_file.write_text(response)
        print(f"💾 Response saved to {debug_file}")
        return False, "none"
    
    print(f"📝 Updating: {file_path}")
    
    if not apply_code_changes(repo_root, file_path, new_content):
        print("❌ Failed to apply changes")
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
    
    # MANDATORY POST-VALIDATION: Ensure project still builds AFTER changes
    if not ensure_builds_or_fix(repo_root, config, "POST-CHANGE"):
        print("❌ CRITICAL: Changes broke the build and could not be fixed automatically.")
        print("   Rolling back changes...")
        # Rollback all affected files
        rollback_changes(repo_root, item.files_affected if item.files_affected else [file_path])
        # Mark as failed so orchestrator skips it and moves to next item
        todo_list.mark_failed(item_id)
        todo_list.save()
        return False, size_category
    
    # Mark as completed
    todo_list.mark_completed(item_id)
    todo_list.save()
    record_last_change(repo_root, "features", item_id, [file_path])
    
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
