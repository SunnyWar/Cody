"""
Refactoring Executor Agent

Executes a specific refactoring task from the TODO list.
"""

import os
import sys
import json
import subprocess
from datetime import datetime
from pathlib import Path
from openai import OpenAI
from todo_manager import TodoList
from executor_state import record_last_change
from validation import ensure_builds_or_fix, rollback_changes


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
    """Load the refactoring execution prompt."""
    repo_root = Path(__file__).parent.parent
    prompt_path = repo_root / ".github" / "ai" / "prompts" / "refactoring_execution.md"
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


def call_ai(prompt: str, config: dict) -> str:
    """Call the AI with the prompt."""
    if config.get("use_local"):
        client = OpenAI(
            api_key="ollama", 
            base_url=config.get("api_base", "http://localhost:11434/v1"),
            timeout=3600.0
        )
    else:
        api_key = os.environ.get("OPENAI_API_KEY")
        if not api_key:
            print(f"\n❌ Error: OPENAI_API_KEY environment variable not set")
            print(f"\n   Set your API key:")
            print(f"   export OPENAI_API_KEY=sk-...")
            print(f"\n   Or configure 'use_local': true in config.json to use a local LLM.\n")
            sys.exit(1)
        client = OpenAI(api_key=api_key, timeout=3600.0)
    
    model = config["model"]
    print(f"🤖 Implementing with {model}...")
    
    response = client.chat.completions.create(
        model=model,
        messages=[
            {"role": "system", "content": "You are a senior Rust engineer implementing refactorings."},
            {"role": "user", "content": prompt}
        ],
        temperature=0.2
    )
    
    return response.choices[0].message.content


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
    """Run tests to validate the refactoring."""
    print("\n🛡️ Validating refactoring...")
    
    steps = [
        ("Format check", ["cargo", "fmt", "--", "--check"]),
        ("Build", ["cargo", "build", "--release"]),
        ("Test", ["cargo", "test"]),
    ]
    
    for step_name, command in steps:
        print(f"\n  Running: {step_name}...")
        result = subprocess.run(
            command,
            cwd=repo_root,
            capture_output=True,
            text=True
        )
        
        if result.returncode != 0:
            print(f"  ❌ {step_name} failed:")
            print(result.stderr)
            return False
        else:
            print(f"  ✅ {step_name} passed")
    
    return True


def execute_refactoring(item_id: str, repo_root: Path, config: dict) -> bool:
    """Execute a specific refactoring task."""
    print("=" * 60)
    print(f"EXECUTING REFACTORING: {item_id}")
    print("=" * 60)
    
    # Load TODO list and find the item
    todo_list = TodoList("refactoring", repo_root)
    
    item = None
    for todo_item in todo_list.items:
        if todo_item.id == item_id:
            item = todo_item
            break
    
    if not item:
        print(f"❌ Item {item_id} not found in TODO list")
        return False
    
    if item.status == "completed":
        print(f"⏭️ Item {item_id} is already completed")
        return True
    
    # Mark as in progress
    todo_list.mark_in_progress(item_id)
    todo_list.save()
    
    # MANDATORY PRE-VALIDATION: Ensure project builds BEFORE making changes
    if not ensure_builds_or_fix(repo_root, config, "PRE-CHANGE"):
        print("❌ CRITICAL: Project does not build before changes and could not be fixed.")
        print("   Aborting to prevent further damage.")
        todo_list.mark_failed(item_id)
        todo_list.save()
        return False
    
    # Build the prompt
    prompt_template = get_prompt_template()
    
    refactoring_details = f"""
ID: {item.id}
Title: {item.title}
Priority: {item.priority}
Category: {item.category}
Description: {item.description}
Proposed Solution: {item.metadata.get('proposed_solution', 'See description')}
Files Affected: {', '.join(item.files_affected)}
"""
    
    # Gather relevant code
    code_context = ""
    if item.files_affected:
        code_context = gather_relevant_code(repo_root, item.files_affected)
    else:
        # Gather all code if no specific files
        for rs_file in repo_root.rglob("*.rs"):
            if "target" not in str(rs_file) and "flycheck" not in str(rs_file):
                rel_path = rs_file.relative_to(repo_root)
                content = rs_file.read_text(encoding='utf-8')
                code_context += f"\n// ========== FILE: {rel_path} ==========\n{content}"
    
    full_prompt = prompt_template.replace("{REFACTORING_DETAILS}", refactoring_details)
    full_prompt += f"\n\n## CODE CONTEXT\n\n{code_context}"
    
    # Call AI with error handling
    try:
        response = call_ai(full_prompt, config)
    except Exception as e:
        logs_dir = repo_root / ".orchestrator_logs"
        logs_dir.mkdir(parents=True, exist_ok=True)
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        error_path = logs_dir / f"refactoring_exec_ai_error_{timestamp}.txt"
        with error_path.open("w", encoding="utf-8", errors="replace") as f:
            f.write(f"AI API Error: {type(e).__name__}\n")
            f.write(f"Error message: {str(e)}\n")
            f.write(f"\nitem_id: {item_id}\n")
        print(f"❌ AI API Error: {type(e).__name__}: {str(e)[:200]}")
        print(f"📄 Error details saved to: {error_path}")
        return False
    
    # Extract and apply changes
    file_path, new_content = extract_file_content(response)
    
    if not file_path or not new_content:
        print("❌ Failed to extract file content from response")
        # Save response for debugging
        debug_file = repo_root / "temp_refactoring_response.txt"
        debug_file.write_text(response)
        print(f"💾 Response saved to {debug_file}")
        return False
    
    print(f"📝 Updating: {file_path}")
    
    if not apply_code_changes(repo_root, file_path, new_content):
        print("❌ Failed to apply changes")
        return False
    
    # MANDATORY POST-VALIDATION: Ensure project still builds AFTER changes
    if not ensure_builds_or_fix(repo_root, config, "POST-CHANGE"):
        print("❌ CRITICAL: Changes broke the build and could not be fixed automatically.")
        print("   Rolling back changes...")
        rollback_changes(repo_root, [file_path])
        # Mark as failed so orchestrator skips it and moves to next item
        todo_list.mark_failed(item_id)
        todo_list.save()
        return False
    
    # Mark as completed
    todo_list.mark_completed(item_id)
    todo_list.save()
    record_last_change(repo_root, "refactoring", item_id, [file_path])
    
    print(f"\n✅ Refactoring {item_id} completed successfully")
    return True


def main():
    """Main entry point."""
    import sys
    
    if len(sys.argv) < 2:
        print("Usage: python refactoring_executor.py <item_id>")
        print("   or: python refactoring_executor.py next")
        sys.exit(1)
    
    config = load_config()
    repo_root = Path(__file__).parent.parent
    
    item_id = sys.argv[1]
    
    if item_id == "next":
        # Get next item
        todo_list = TodoList("refactoring", repo_root)
        next_item = todo_list.get_next_item()
        if not next_item:
            print("No items available to work on")
            sys.exit(0)
        item_id = next_item.id
        print(f"Working on next item: {item_id}")
    
    success = execute_refactoring(item_id, repo_root, config)
    
    if success:
        print(f"\n{'=' * 60}")
        print("Refactoring completed successfully")
        print(f"{'=' * 60}")
    else:
        print(f"\n{'=' * 60}")
        print("Refactoring failed")
        print(f"{'=' * 60}")
        sys.exit(1)


if __name__ == "__main__":
    main()
