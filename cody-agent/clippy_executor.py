"""
Clippy Executor Agent

Executes a specific clippy warning fix from the TODO list using an LLM.
"""

import os
import sys
import json
import subprocess
from pathlib import Path
from openai import OpenAI
from todo_manager import TodoList
from validation import ensure_builds_or_fix, rollback_changes


def load_config():
    """Load configuration."""
    config_path = Path(__file__).parent / "config.json"
    
    if not config_path.exists():
        print(f"❌ Error: Configuration file not found at {config_path}")
        sys.exit(1)
    
    try:
        with open(config_path) as f:
            config = json.load(f)
        if not config:
            raise ValueError("Config file is empty")
        return config
    except Exception as e:
        print(f"❌ Error reading config file: {e}")
        sys.exit(1)


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
            print("\n❌ Error: OPENAI_API_KEY environment variable not set")
            print("\n   Set your API key:")
            print("   export OPENAI_API_KEY=sk-...")
            print("\n   Or configure 'use_local': true in config.json to use a local LLM.\n")
            sys.exit(1)
        client = OpenAI(api_key=api_key, timeout=3600.0)

    model = config["model"]
    print(f"🤖 Fixing clippy warning with {model}...")

    response = client.chat.completions.create(
        model=model,
        messages=[
            {
                "role": "system",
                "content": "You are a senior Rust engineer. You MUST return the COMPLETE, FULL file with ALL code included. NEVER use placeholders like '...' or comments like '// rest of code unchanged'. Return only a single ```rust code block with the file path as the first comment."
            },
            {"role": "user", "content": prompt}
        ],
        temperature=0.2
    )

    return response.choices[0].message.content


def extract_file_content(response: str) -> tuple[str, str]:
    """Extract file path and content from LLM response.
    Returns (file_path, content) or (None, None) if extraction fails.
    """
    if "```rust" in response:
        start = response.find("```rust") + 7
        end = response.find("```", start)
        if end != -1:
            code = response[start:end].strip()
            
            # Reject responses with placeholder markers
            placeholder_markers = [
                "...",
                "existing code",
                "rest of the code",
                "unchanged",
                "// (rest",
                "// ..."
            ]
            code_lower = code.lower()
            for marker in placeholder_markers:
                if marker in code_lower:
                    print(f"⚠️ Response contains placeholder marker: '{marker}'")
                    return None, None
            
            lines = code.split("\n")
            file_path = None
            for line in lines[:5]:
                if "//" in line and (".rs" in line or "/" in line):
                    path_part = line.split("//", 1)[1].strip()
                    if path_part and not path_part.startswith(" "):
                        file_path = path_part
                    break

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

        full_path.write_text(new_content, encoding="utf-8")
        print(f"✅ Updated {file_path}")
        return True

    except Exception as e:
        print(f"❌ Error writing file: {e}")
        return False


def apply_direct_fixes(repo_root: Path, item) -> bool:
    """Apply fixes directly using suggestions from TODO item."""
    suggestions = item.metadata.get("suggestions", [])
    if not suggestions:
        print("⚠️ No suggestions found in TODO item")
        return False
    
    file_path = repo_root / item.metadata.get("file", "")
    if not file_path.exists():
        print(f"❌ File not found: {file_path}")
        return False
    
    try:
        content = file_path.read_text(encoding="utf-8")
        original_content = content
        applied_count = 0
        
        for suggestion in suggestions:
            old_text = suggestion.get("suggestion", "")
            new_text = suggestion.get("replacement", "")
            
            if old_text and new_text and old_text in content:
                content = content.replace(old_text, new_text, 1)  # Replace only first occurrence
                applied_count += 1
                print(f"  ✓ {old_text[:60]}... → {new_text[:60]}...")
        
        if content != original_content:
            file_path.write_text(content, encoding="utf-8")
            rel_path = file_path.relative_to(repo_root)
            print(f"✅ Applied {applied_count} fix(es) to {rel_path}")
            return True
        else:
            print(f"⚠️ No changes made - suggestions not found in file")
            return False
            
    except Exception as e:
        print(f"❌ Error applying fixes: {e}")
        return False


def execute_clippy_fix(item_id: str, repo_root: Path, config: dict) -> bool:
    """Execute a specific clippy fix using the LLM."""
    print("=" * 60)
    print(f"EXECUTING CLIPPY FIX: {item_id}")
    print("=" * 60)

    todo_list = TodoList("clippy", repo_root)

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

    todo_list.mark_in_progress(item_id)
    todo_list.save()

    # MANDATORY PRE-VALIDATION: Ensure project builds BEFORE making changes
    if not ensure_builds_or_fix(repo_root, config, "PRE-CHANGE"):
        print("❌ CRITICAL: Project does not build before changes and could not be fixed.")
        print("   Aborting to prevent further damage.")
        return False

    file_path = item.metadata.get("file")
    if not file_path and item.files_affected:
        file_path = item.files_affected[0]

    if not file_path:
        print("❌ No file path available for this clippy item")
        return False

    full_path = repo_root / file_path
    if not full_path.exists():
        print(f"❌ File not found: {full_path}")
        return False

    file_content = full_path.read_text(encoding="utf-8")
    lint_name = item.metadata.get("lint_name", "clippy")
    line = item.metadata.get("line", 0)
    column = item.metadata.get("column", 0)
    rendered = item.metadata.get("rendered", item.description)

    prompt = (
        f"Fix the following Clippy warning by editing the file.\n\n"
        f"Warning:\n"
        f"- Lint: {lint_name}\n"
        f"- File: {file_path}\n"
        f"- Line: {line}\n"
        f"- Column: {column}\n"
        f"- Message: {rendered}\n\n"
        f"CRITICAL INSTRUCTIONS:\n"
        f"- Return a SINGLE ```rust code block\n"
        f"- First line must be: // {file_path}\n"
        f"- Include the COMPLETE, FULL file with ALL code\n"
        f"- NEVER use '...' or placeholder comments\n"
        f"- NEVER omit any code\n"
        f"- Make ONLY the minimal change to fix the Clippy warning\n\n"
        f"Current file content:\n\n{file_content}\n"
    )

    response = call_ai(prompt, config)
    
    # Save response for debugging
    from datetime import datetime
    logs_dir = repo_root / ".orchestrator_logs"
    logs_dir.mkdir(parents=True, exist_ok=True)
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    response_path = logs_dir / f"clippy_llm_response_{item_id}_{timestamp}.txt"
    with response_path.open("w", encoding="utf-8", errors="replace") as f:
        f.write(response)
    print(f"📄 LLM response saved to: {response_path}")
    
    response_file_path, new_content = extract_file_content(response)

    if not response_file_path or not new_content:
        print("❌ LLM response did not include updated file content")
        print(f"   Response preview: {response[:500]}...")
        if item.metadata.get("suggestions"):
            print("📝 Falling back to pre-vetted fixes from TODO item...")
            if apply_direct_fixes(repo_root, item):
                todo_list.mark_completed(item_id)
                todo_list.save()
                print(f"\n✅ Clippy fix {item_id} completed successfully")
                return True
        return False
    
    # Validate content length (should be similar to original)
    original_lines = len(file_content.split("\n"))
    new_lines = len(new_content.split("\n"))
    if new_lines < original_lines * 0.5:
        print(f"❌ LLM response is suspiciously short: {new_lines} lines vs {original_lines} original")
        print("   This suggests the LLM used placeholders instead of returning full file")
        return False

    if response_file_path != file_path:
        print(f"⚠️ LLM returned a different file path: {response_file_path}")

    if not apply_code_changes(repo_root, file_path, new_content):
        print("❌ Failed to apply code changes")
        return False

    # MANDATORY POST-VALIDATION: Ensure project still builds AFTER changes
    if not ensure_builds_or_fix(repo_root, config, "POST-CHANGE"):
        print("❌ CRITICAL: Changes broke the build and could not be fixed automatically.")
        print("   Rolling back changes...")
        rollback_changes(repo_root, [file_path])
        return False

    # Only mark complete if changes were applied AND build is successful
    todo_list.mark_completed(item_id)
    todo_list.save()
    print(f"\n✅ Clippy fix {item_id} completed successfully")
    return True


def main():
    """Main entry point."""
    import sys

    if len(sys.argv) < 2:
        print("Usage: python clippy_executor.py <item_id>")
        print("   or: python clippy_executor.py next")
        sys.exit(1)

    config = load_config()
    repo_root = Path(__file__).parent.parent

    item_id = sys.argv[1]

    if item_id == "next":
        todo_list = TodoList("clippy", repo_root)
        next_item = todo_list.get_next_item()
        if not next_item:
            print("No items available to work on")
            sys.exit(0)
        item_id = next_item.id
        print(f"Working on next item: {item_id}")

    success = execute_clippy_fix(item_id, repo_root, config)

    if success:
        print(f"\n{'=' * 60}")
        print("Clippy fix completed successfully")
        print(f"{'=' * 60}")
    else:
        print(f"\n{'=' * 60}")
        print("Clippy fix failed")
        print(f"{'=' * 60}")
        sys.exit(1)


if __name__ == "__main__":
    main()
