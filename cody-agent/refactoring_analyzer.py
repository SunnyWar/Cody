"""
Refactoring Analysis Agent

Analyzes the Cody chess engine codebase for refactoring opportunities.
"""

import sys
import json
import re
from datetime import datetime
from pathlib import Path
from todo_manager import TodoList, generate_unique_id
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


def get_repo_root() -> Path:
    """Dynamically resolve the repository root."""
    current_file = Path(__file__).resolve()
    repo_root = current_file.parent.parent

    # Ensure the resolved path contains the expected structure
    if not (repo_root / "Cargo.toml").exists():
        print("❌ Error: Unable to locate repository root. Ensure the script is within the Cody repository.")
        sys.exit(1)

    return repo_root


def get_prompt_template():
    """Load the refactoring analysis prompt."""
    repo_root = get_repo_root()
    prompt_path = repo_root / ".github" / "ai" / "prompts" / "refactoring_analysis.md"
    return prompt_path.read_text(encoding='utf-8')


def gather_code_context(repo_root: Path) -> str:
    """Gather selected Rust source code for analysis."""
    # Dynamically detect relevant Rust files
    excluded_dirs = {"target", "flycheck"}
    code_context = []

    for rs_file in repo_root.rglob("*.rs"):
        if any(excluded_dir in rs_file.parts for excluded_dir in excluded_dirs):
            continue

        rel_path = rs_file.relative_to(repo_root)
        content = rs_file.read_text(encoding='utf-8')
        code_context.append(f"\n// ========== FILE: {rel_path} ==========\n{content}")

    if not code_context:
        print("❌ Error: No relevant Rust files found or code context is empty.")
        print("   Ensure the repository contains the expected files and paths.")
        sys.exit(1)

    return "\n".join(code_context)


def call_ai(prompt: str, config: dict, repo_root: Path) -> str:
    """Call the agent with the prompt."""
    model = config.get("model")
    if model:
        print(f"🤖 Analyzing with {model}...")
    else:
        print("🤖 Analyzing...")

    system_prompt = (
        "You are a senior Rust architect analyzing code for refactoring opportunities. "
        "You MUST respond with ONLY valid JSON. Wrap the output in a root key 'items', "
        "e.g., {\"items\": [...]}. Do not include any text, explanations, or markdown formatting - "
        "only output the raw JSON object."
    )

    return run_agent(system_prompt, prompt, config, repo_root, "refactoring_analysis")


def extract_json_from_response(response: str, repo_root: Path, phase: str) -> list:
    """Extract JSON array from AI response using multiple strategies."""
    json_str = None

    # Log the raw response for debugging
    logs_dir = repo_root / ".orchestrator_logs"
    logs_dir.mkdir(parents=True, exist_ok=True)
    raw_response_path = logs_dir / "debug_raw_response.txt"
    with raw_response_path.open("w", encoding="utf-8", errors="replace") as f:
        f.write(response)

    # Strategy 1: Try to find JSON in ```json code blocks
    match = re.search(r"```json\s*([\s\S]*?)\s*```", response, re.DOTALL)
    if match:
        json_str = match.group(1).strip()

    # Strategy 2: Try generic code blocks (``` ... ```)
    if not json_str:
        match = re.search(r"```\s*([\s\S]*?)\s*```", response, re.DOTALL)
        if match:
            candidate = match.group(1).strip()
            # Only use if it looks like JSON (starts with [ or {)
            if candidate.startswith('[') or candidate.startswith('{'):
                json_str = candidate

    # Strategy 3: Look for JSON array anywhere in response (handles embedded JSON)
    if not json_str:
        match = re.search(r'(\[\s*(?:{[\s\S]*?}\s*,?\s*)*\])', response, re.DOTALL)
        if match:
            json_str = match.group(1).strip()

    # Strategy 4: Try the whole response if it starts with [ or {
    if not json_str:
        trimmed = response.strip()
        if trimmed.startswith('[') or trimmed.startswith('{'):
            json_str = trimmed

    # If all strategies fail and response doesn't look like JSON, assume empty result
    if not json_str:
        print(f"⚠️ AI did not return JSON format. Response preview: {response[:200]}...")
        print(f"   Returning empty list. This might mean no {phase} items were found.")
        return []

    try:
        result = json.loads(json_str)
        # Handle new object structure with "items" key
        if isinstance(result, dict) and "items" in result:
            return result.get("items", [])
        # Ensure we return a list for backward compatibility
        return result if isinstance(result, list) else []
    except json.JSONDecodeError as e:
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        dump_path = logs_dir / f"{phase}_parse_fail_{timestamp}.txt"
        with dump_path.open("w", encoding="utf-8", errors="replace") as f:
            f.write("JSON parse failure\n")
            f.write(f"Error: {e}\n")
            f.write("\n=== Attempted JSON String ===\n")
            f.write(json_str)
            f.write("\n\n=== Full Response ===\n")
            f.write(response)
        print(f"❌ Failed to parse JSON: {e}")
        print(f"Response preview: {response[:500]}...")
        print(f"📄 Full response saved to: {dump_path}")
        print(f"⚠️ Returning empty list to continue workflow")
        # Don't raise - return empty list to allow workflow to continue
        return []


def analyze(repo_root: Path, config: dict) -> int:
    """Run refactoring analysis and update TODO list."""
    print("=" * 60)
    print("REFACTORING ANALYSIS")
    print("=" * 60)
    
    # Load existing TODO list
    todo_list = TodoList("refactoring", repo_root)
    existing_ids = todo_list.get_all_ids()
    
    # Build the prompt
    prompt_template = get_prompt_template()
    code_context = gather_code_context(repo_root)
    
    # Include existing TODO items in prompt to avoid duplicates
    existing_todos_info = ""
    if todo_list.items:
        existing_todos_info = "\n## Existing TODO Items (DO NOT DUPLICATE)\n\n"
        for item in todo_list.items:
            existing_todos_info += f"- {item.id}: {item.title} [{item.status}]\n"
    
    full_prompt = f"{prompt_template}\n\n{existing_todos_info}\n\n## CODE TO ANALYZE\n\n{code_context}"
    
    # Call AI with error handling
    try:
        response = call_ai(full_prompt, config, repo_root)
    except Exception as e:
        logs_dir = repo_root / ".orchestrator_logs"
        logs_dir.mkdir(parents=True, exist_ok=True)
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        error_path = logs_dir / f"refactoring_ai_error_{timestamp}.txt"
        with error_path.open("w", encoding="utf-8", errors="replace") as f:
            f.write(f"AI API Error: {type(e).__name__}\n")
            f.write(f"Error message: {str(e)}\n")
            f.write("\n=== Prompt sent ===\n")
            f.write(full_prompt[:2000])  # First 2000 chars of prompt
        print(f"❌ AI API Error: {type(e).__name__}: {str(e)[:200]}")
        print(f"📄 Error details saved to: {error_path}")
        raise RuntimeError(f"refactoring AI call failed; see {error_path}")
    
    # Parse response
    new_items = extract_json_from_response(response, repo_root, "refactoring")
    
    # Validate that new_items is a list of dicts
    if not isinstance(new_items, list):
        print(f"⚠️ Expected list of items, got {type(new_items).__name__}")
        return 0
    
    # Filter out non-dict items
    new_items = [item for item in new_items if isinstance(item, dict)]
    if not new_items:
        print("⚠️ No refactoring opportunities found or failed to parse response")
        return 0
    
    # Generate unique IDs and add items
    for item in new_items:
        if "id" not in item or not item["id"]:
            existing_new_ids = [i.get("id") for i in new_items if isinstance(i, dict) and i.get("id")]
            item["id"] = generate_unique_id("refactoring", existing_ids + existing_new_ids)
    
    # Add to TODO list
    added = todo_list.add_items(new_items, check_duplicates=True)
    
    # Save
    if added > 0:
        todo_list.save()
        print(f"\n✅ Added {added} new refactoring opportunities to TODO list")
    else:
        print("\n⏭️ No new items added (all were duplicates)")
    
    return added


def main():
    """Main entry point."""
    config = load_config()
    repo_root = get_repo_root()
    
    added = analyze(repo_root, config)
    
    print(f"\n{'=' * 60}")
    print(f"Analysis complete: {added} new items added")
    print(f"{'=' * 60}")


if __name__ == "__main__":
    main()
