"""
TODO Management Utilities for Cody AI Agent System

Handles loading, saving, validating, and manipulating TODO lists
for refactoring, performance, and feature tasks.
"""

import json
from pathlib import Path
from typing import List, Dict, Any, Optional
from datetime import datetime


class TodoItem:
    """Represents a single TODO item."""
    
    def __init__(self, data: Dict[str, Any]):
        self.id = data["id"]
        self.title = data["title"]
        self.priority = data.get("priority", "medium")
        self.category = data.get("category", "")
        self.description = data.get("description", "")
        self.status = data.get("status", "not-started")  # not-started, in-progress, completed
        self.created_at = data.get("created_at", datetime.now().isoformat())
        self.completed_at = data.get("completed_at", None)
        self.estimated_complexity = data.get("estimated_complexity", "medium")
        self.files_affected = data.get("files_affected", [])
        self.dependencies = data.get("dependencies", [])
        self.metadata = {k: v for k, v in data.items() 
                        if k not in ["id", "title", "priority", "category", 
                                    "description", "status", "created_at", 
                                    "completed_at", "estimated_complexity",
                                    "files_affected", "dependencies"]}
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            "id": self.id,
            "title": self.title,
            "priority": self.priority,
            "category": self.category,
            "description": self.description,
            "status": self.status,
            "created_at": self.created_at,
            "completed_at": self.completed_at,
            "estimated_complexity": self.estimated_complexity,
            "files_affected": self.files_affected,
            "dependencies": self.dependencies,
            **self.metadata
        }
    
    def mark_completed(self):
        """Mark item as completed."""
        self.status = "completed"
        self.completed_at = datetime.now().isoformat()
    
    def mark_in_progress(self):
        """Mark item as in progress."""
        self.status = "in-progress"
    
    def is_duplicate(self, other: 'TodoItem') -> bool:
        """Check if this item is a duplicate of another."""
        # Check title similarity (simple string match)
        if self.title.lower() == other.title.lower():
            return True

        # Check file overlap and similar category
        if (
            self.category == other.category and 
            self.files_affected and other.files_affected and
            set(self.files_affected) == set(other.files_affected)
        ):
            return True

        return False


class TodoList:
    """Manages a TODO list for a specific category."""
    
    def __init__(self, category: str, repo_root: Path):
        self.category = category  # refactoring, performance, features
        self.repo_root = Path(repo_root)
        self.file_path = self.repo_root / f"TODO_{category.upper()}.md"
        self.json_path = self.repo_root / f".todo_{category}.json"
        self.items: List[TodoItem] = []
        self.load()
    
    def load(self):
        """Load TODO list from JSON file."""
        if self.json_path.exists():
            try:
                with open(self.json_path, 'r') as f:
                    data = json.load(f)
                    self.items = [TodoItem(item) for item in data]
                    print(f"✅ Loaded {len(self.items)} items from {self.json_path}")
            except Exception as e:
                print(f"⚠️ Error loading {self.json_path}: {e}")
                self.items = []
        else:
            print(f"📝 No existing TODO list found at {self.json_path}, starting fresh")
    
    def save(self):
        """Save TODO list to both JSON and Markdown."""
        # Remove completed items
        self.items = [item for item in self.items if item.status != "completed"]

        # Remove duplicates
        unique_items = []
        for item in self.items:
            if not any(item.is_duplicate(existing) for existing in unique_items):
                unique_items.append(item)
        self.items = unique_items

        # Save JSON for programmatic access
        with open(self.json_path, 'w') as f:
            json.dump([item.to_dict() for item in self.items], f, indent=2)
        
        # Save Markdown for human readability
        self._save_markdown()
        
        print(f"💾 Saved {len(self.items)} items to {self.json_path} and {self.file_path}")
    
    def _save_markdown(self):
        """Generate markdown representation."""
        lines = [
            f"# TODO List: {self.category.title()}",
            f"\nGenerated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}",
            f"\n**Stats**: {len(self.items)} total | "
            f"{self.count_by_status('not-started')} not started | "
            f"{self.count_by_status('in-progress')} in progress | "
            f"{self.count_by_status('completed')} completed\n",
            "---\n"
        ]
        
        # Group by status
        for status in ["in-progress", "not-started", "completed"]:
            status_items = [item for item in self.items if item.status == status]
            if status_items:
                lines.append(f"\n## {status.replace('-', ' ').title()}\n")
                for item in status_items:
                    check = "x" if item.status == "completed" else " "
                    lines.append(f"\n### [{check}] {item.id}: {item.title}\n")
                    lines.append(f"- **Priority**: {item.priority}\n")
                    lines.append(f"- **Category**: {item.category}\n")
                    lines.append(f"- **Complexity**: {item.estimated_complexity}\n")
                    if item.files_affected:
                        lines.append(f"- **Files**: {', '.join(item.files_affected)}\n")
                    if item.dependencies:
                        lines.append(f"- **Dependencies**: {', '.join(item.dependencies)}\n")
                    lines.append(f"\n{item.description}\n")
                    if item.completed_at:
                        lines.append(f"\n*Completed: {item.completed_at}*\n")
        
        with open(self.file_path, 'w') as f:
            f.writelines(lines)
    
    def add_items(self, new_items: List[Dict[str, Any]], check_duplicates: bool = True) -> int:
        """Add new items, optionally checking for duplicates."""
        added = 0
        for item_data in new_items:
            new_item = TodoItem(item_data)
            
            if check_duplicates:
                is_dup = any(new_item.is_duplicate(existing) for existing in self.items)
                if is_dup:
                    print(f"⏭️ Skipping duplicate: {new_item.id} - {new_item.title}")
                    continue
            
            self.items.append(new_item)
            added += 1
            print(f"➕ Added: {new_item.id} - {new_item.title}")
        
        return added
    
    def get_next_item(self) -> Optional[TodoItem]:
        """Get the next not-started item with highest priority."""
        not_started = [item for item in self.items if item.status == "not-started"]
        
        if not not_started:
            return None
        
        # Check dependencies
        available = []
        for item in not_started:
            if not item.dependencies:
                available.append(item)
            else:
                # Check if all dependencies are completed
                deps_completed = all(
                    any(i.id == dep and i.status == "completed" for i in self.items)
                    for dep in item.dependencies
                )
                if deps_completed:
                    available.append(item)
        
        if not available:
            print("⚠️ All remaining items have unmet dependencies")
            return None
        
        # Sort by priority
        priority_order = {"critical": 0, "high": 1, "medium": 2, "low": 3}
        available.sort(key=lambda x: priority_order.get(x.priority, 2))
        
        return available[0]
    
    def mark_completed(self, item_id: str):
        """Mark an item as completed."""
        for item in self.items:
            if item.id == item_id:
                item.mark_completed()
                print(f"✅ Marked completed: {item_id}")
                return True
        print(f"❌ Item not found: {item_id}")
        return False
    
    def mark_in_progress(self, item_id: str):
        """Mark an item as in progress."""
        for item in self.items:
            if item.id == item_id:
                item.mark_in_progress()
                print(f"🔄 Marked in progress: {item_id}")
                return True
        print(f"❌ Item not found: {item_id}")
        return False
    
    def count_by_status(self, status: str) -> int:
        """Count items by status."""
        return sum(1 for item in self.items if item.status == status)
    
    def get_all_ids(self) -> List[str]:
        """Get all item IDs."""
        return [item.id for item in self.items]
    
    def validate_and_clean(self) -> int:
        """Validate all items are still relevant, remove obsolete ones."""
        # This would need code analysis to determine if items are still valid
        # For now, just a placeholder
        print(f"🔍 Validating {len(self.items)} items...")
        # TODO: Add validation logic based on current codebase state
        return 0


def generate_unique_id(category: str, existing_ids: List[str]) -> str:
    """Generate a unique ID for a new TODO item."""
    prefix_map = {
        "refactoring": "REF",
        "performance": "PERF",
        "features": "FEAT",
        "clippy": "CLIP"
    }
    prefix = prefix_map.get(category, "TODO")
    
    # Extract numbers from existing IDs with same prefix
    numbers = []
    for id_str in existing_ids:
        if id_str.startswith(prefix):
            try:
                num = int(id_str.split('-')[1])
                numbers.append(num)
            except (IndexError, ValueError):
                pass
    
    next_num = max(numbers) + 1 if numbers else 1
    return f"{prefix}-{next_num:03d}"


if __name__ == "__main__":
    # Test the module
    import sys
    repo_root = Path(__file__).parent.parent
    
    if len(sys.argv) > 1:
        category = sys.argv[1]
        todo = TodoList(category, repo_root)
        print(f"\n{category.title()} TODO List:")
        print(f"Total: {len(todo.items)}")
        print(f"Not started: {todo.count_by_status('not-started')}")
        print(f"In progress: {todo.count_by_status('in-progress')}")
        print(f"Completed: {todo.count_by_status('completed')}")
        
        next_item = todo.get_next_item()
        if next_item:
            print(f"\nNext item to work on: {next_item.id} - {next_item.title}")
    else:
        print("Usage: python todo_manager.py <category>")
        print("Categories: refactoring, performance, features")
