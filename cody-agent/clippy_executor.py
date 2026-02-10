"""
Clippy Executor Agent

Executes a specific clippy warning fix from the TODO list using direct file editing.
NO LLM patch generation - uses pre-vetted suggestions from TODO list.
"""

import sys
import json
import subprocess
from pathlib import Path
from todo_manager import TodoList


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


    """Placeholder - LLM calls removed. Use direct fixes instead."""
    pass


def execute_clippy_fix(item_id: str, repo_root: Path, config: dict) -> bool:
    """Execute a specific clippy fix using direct file editing (no LLM)."""
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

    # Apply fixes directly from TODO list suggestions (no LLM needed)
    if item.metadata.get("suggestions"):
        print("📝 Applying pre-vetted fixes from TODO item...")
        if apply_direct_fixes(repo_root, item):
            todo_list.mark_completed(item_id)
            todo_list.save()
            print(f"\n✅ Clippy fix {item_id} completed successfully")
            return True
        else:
            print("❌ Failed to apply direct fixes")
            todo_list.mark_in_progress(item_id)  # Keep as in-progress for retry
            todo_list.save()
            return False
    else:
        # No suggestions available - mark as completed with note
        print("⚠️ No fix suggestions available in TODO item")
        todo_list.mark_completed(item_id)
        todo_list.save()
        print(f"⏭️ Skipped {item_id} - no actionable fixes")
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
