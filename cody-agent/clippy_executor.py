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
from console_utils import safe_print
from validation import ensure_builds_or_fix, rollback_changes
from datetime import datetime


def load_config():
    """Load configuration."""
    config_path = Path(__file__).parent / "config.json"
    
    if not config_path.exists():
        safe_print(f"❌ Error: Configuration file not found at {config_path}")
        sys.exit(1)
    
    try:
        with open(config_path) as f:
            config = json.load(f)
        if not config:
            raise ValueError("Config file is empty")
        return config
    except Exception as e:
        safe_print(f"❌ Error reading config file: {e}")
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
            safe_print("\n❌ Error: OPENAI_API_KEY environment variable not set")
            safe_print("\n   Set your API key:")
            safe_print("   export OPENAI_API_KEY=sk-...")
            safe_print("\n   Or configure 'use_local': true in config.json to use a local LLM.\n")
            sys.exit(1)
        client = OpenAI(api_key=api_key, timeout=3600.0)

    model = config["model"]
    safe_print(f"🤖 Fixing clippy warning with {model}...")

    response = client.chat.completions.create(
        model=model,
        messages=[
            {
                "role": "system",
                "content": "You are a senior Rust engineer. You MUST return the COMPLETE, FULL file with ALL code included. NEVER use placeholders like '...' or comments like '// rest of code unchanged'. Return only a single ```rust code block with the file path as the first comment. Focus exclusively on the provided Clippy diagnostic and do not fix other warnings."
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
                    safe_print(f"⚠️ Response contains placeholder marker: '{marker}'")
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

    safe_print("⚠️ No clear code block found in response")
    return None, None


def apply_code_changes(repo_root: Path, file_path: str, new_content: str) -> bool:
    """Write new content directly to file."""
    try:
        full_path = repo_root / file_path
        if not full_path.parent.exists():
            safe_print(f"❌ Parent directory does not exist: {full_path.parent}")
            return False

        full_path.write_text(new_content, encoding="utf-8")
        safe_print(f"✅ Updated {file_path}")
        return True

    except Exception as e:
        safe_print(f"❌ Error writing file: {e}")
        return False


def apply_direct_fixes(repo_root: Path, item) -> bool:
    """Apply fixes directly using suggestions from TODO item."""
    suggestions = item.metadata.get("suggestions", [])
    if not suggestions:
        safe_print("⚠️ No suggestions found in TODO item")
        return False
    
    file_path = repo_root / item.metadata.get("file", "")
    if not file_path.exists():
        safe_print(f"❌ File not found: {file_path}")
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
                safe_print(f"  ✓ {old_text[:60]}... → {new_text[:60]}...")
        
        if content != original_content:
            file_path.write_text(content, encoding="utf-8")
            rel_path = file_path.relative_to(repo_root)
            safe_print(f"✅ Applied {applied_count} fix(es) to {rel_path}")
            return True
        else:
            safe_print(f"⚠️ No changes made - suggestions not found in file")
            return False
            
    except Exception as e:
        safe_print(f"❌ Error applying fixes: {e}")
        return False


def mark_infeasible(todo_list: TodoList, item_id: str) -> bool:
    """Mark an item as infeasible to avoid loops."""
    for item in todo_list.items:
        if item.id == item_id:
            item.status = "infeasible"
            item.completed_at = datetime.now().isoformat()
            safe_print(f"Marked infeasible: {item_id}")
            return True
    safe_print(f"❌ Item not found: {item_id}")
    return False


def run_clippy_parser(repo_root: Path) -> tuple[list, int]:
    """Run clippy_parser.py and return (warnings, returncode)."""
    parser_script = repo_root / "cody-agent" / "clippy_parser.py"
    command = [sys.executable, str(parser_script)]
    result = subprocess.run(
        command,
        cwd=repo_root,
        capture_output=True,
        text=True
    )

    warnings = []
    for line in result.stdout.strip().split("\n"):
        if not line.strip():
            continue
        try:
            warnings.append(json.loads(line))
        except json.JSONDecodeError:
            continue

    return warnings, result.returncode


def warning_persists(warnings: list, file_path: str, lint_name: str, line: int) -> bool:
    """Check whether the target warning still appears in clippy output."""
    for warning in warnings:
        message = warning.get("message", {})
        code = message.get("code", {}).get("code", "")
        spans = message.get("spans", [])
        span = spans[0] if spans else {}
        file_name = span.get("file_name", "")
        line_start = int(span.get("line_start", 0) or 0)

        if code != lint_name:
            continue
        if file_name != file_path:
            continue
        if line and line_start and line_start != line:
            continue
        return True
    return False


def run_cargo_test(repo_root: Path) -> bool:
    """Run cargo test to ensure no regressions."""
    result = subprocess.run(
        ["cargo", "test"],
        cwd=repo_root,
        capture_output=True,
        text=True
    )
    if result.returncode != 0:
        safe_print("❌ cargo test failed")
        safe_print(result.stderr[:2000])
        return False
    return True


def execute_clippy_fix(item_id: str, repo_root: Path, config: dict) -> bool:
    """Execute a specific clippy fix using the LLM."""
    safe_print("=" * 60)
    safe_print(f"EXECUTING CLIPPY FIX: {item_id}")
    safe_print("=" * 60)

    todo_list = TodoList("clippy", repo_root)

    item = None
    for todo_item in todo_list.items:
        if todo_item.id == item_id:
            item = todo_item
            break

    if not item:
        safe_print(f"❌ Item {item_id} not found in TODO list")
        return False

    if item.status == "completed":
        safe_print(f"⏭️ Item {item_id} is already completed")
        return True

    todo_list.mark_in_progress(item_id)
    todo_list.save()

    # MANDATORY PRE-VALIDATION: Ensure project builds BEFORE making changes
    if not ensure_builds_or_fix(repo_root, config, "PRE-CHANGE"):
        safe_print("❌ CRITICAL: Project does not build before changes and could not be fixed.")
        safe_print("   Aborting to prevent further damage.")
        todo_list.mark_failed(item_id)
        todo_list.save()
        return False

    file_path = item.metadata.get("file")
    if not file_path and item.files_affected:
        file_path = item.files_affected[0]

    if not file_path:
        safe_print("❌ No file path available for this clippy item")
        return False

    full_path = repo_root / file_path
    if not full_path.exists():
        safe_print(f"❌ File not found: {full_path}")
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
        f"- You are being shown one specific warning among many. Do not attempt to fix other warnings in the file; focus exclusively on the provided diagnostic.\n\n"
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
    safe_print(f"📄 LLM response saved to: {response_path}")
    
    response_file_path, new_content = extract_file_content(response)

    if not response_file_path or not new_content:
        safe_print("❌ LLM response did not include updated file content")
        safe_print(f"   Response preview: {response[:500]}...")
        if item.metadata.get("suggestions"):
            safe_print("📝 Falling back to pre-vetted fixes from TODO item...")
            if apply_direct_fixes(repo_root, item):
                todo_list.mark_completed(item_id)
                todo_list.save()
                safe_print(f"\n✅ Clippy fix {item_id} completed successfully")
                return True
        # LLM failed to provide valid code - mark as failed and skip
        todo_list.mark_failed(item_id)
        todo_list.save()
        return False

    if new_content == file_content:
        safe_print("⚠️ LLM returned identical content; marking as infeasible")
        mark_infeasible(todo_list, item_id)
        todo_list.save()
        return False
    
    # Validate content length (should be similar to original)
    original_lines = len(file_content.split("\n"))
    new_lines = len(new_content.split("\n"))
    if new_lines < original_lines * 0.5:
        safe_print(f"❌ LLM response is suspiciously short: {new_lines} lines vs {original_lines} original")
        safe_print("   This suggests the LLM used placeholders instead of returning full file")
        todo_list.mark_failed(item_id)
        todo_list.save()
        return False

    if response_file_path != file_path:
        safe_print(f"⚠️ LLM returned a different file path: {response_file_path}")

    if not apply_code_changes(repo_root, file_path, new_content):
        safe_print("❌ Failed to apply code changes")
        return False

    warnings, clippy_code = run_clippy_parser(repo_root)
    warning_still_present = warning_persists(warnings, file_path, lint_name, line)

    test_ok = run_cargo_test(repo_root)

    if warning_still_present:
        safe_print("❌ Warning persists after fix. Marking as failed.")
        rollback_changes(repo_root, [file_path])
        todo_list.mark_failed(item_id)
        todo_list.save()
        return False

    if not test_ok:
        safe_print("❌ Changes failed cargo test. Rolling back...")
        rollback_changes(repo_root, [file_path])
        todo_list.mark_failed(item_id)
        todo_list.save()
        return False

    if clippy_code != 0:
        safe_print("⚠️ Clippy still reports other warnings; target warning cleared.")

    # Only mark complete if changes were applied AND build is successful
    todo_list.mark_completed(item_id)
    todo_list.save()
    safe_print(f"\n✅ Clippy fix {item_id} completed successfully")
    return True


def main():
    """Main entry point."""
    import sys

    if len(sys.argv) < 2:
        safe_print("Usage: python clippy_executor.py <item_id>")
        safe_print("   or: python clippy_executor.py next")
        sys.exit(1)

    config = load_config()
    repo_root = Path(__file__).parent.parent

    item_id = sys.argv[1]

    if item_id == "next":
        todo_list = TodoList("clippy", repo_root)
        next_item = todo_list.get_next_item()
        if not next_item:
            safe_print("No items available to work on")
            sys.exit(0)
        item_id = next_item.id
        safe_print(f"Working on next item: {item_id}")

    success = execute_clippy_fix(item_id, repo_root, config)

    if success:
        safe_print(f"\n{'=' * 60}")
        safe_print("Clippy fix completed successfully")
        safe_print(f"{'=' * 60}")
    else:
        safe_print(f"\n{'=' * 60}")
        safe_print("Clippy fix failed")
        safe_print(f"{'=' * 60}")
        sys.exit(1)


if __name__ == "__main__":
    main()
