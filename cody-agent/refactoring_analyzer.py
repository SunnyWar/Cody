"""
Refactoring Analysis Agent

Analyzes the Cody chess engine codebase for refactoring opportunities.
"""

import os
import json
from pathlib import Path
from openai import OpenAI
from todo_manager import TodoList, generate_unique_id


def load_config():
    """Load configuration."""
    config_path = Path(__file__).parent / "config.json"
    with open(config_path) as f:
        return json.load(f)


def get_prompt_template():
    """Load the refactoring analysis prompt."""
    repo_root = Path(__file__).parent.parent
    prompt_path = repo_root / ".github" / "ai" / "prompts" / "refactoring_analysis.md"
    return prompt_path.read_text()


def gather_code_context(repo_root: Path) -> str:
    """Gather all Rust source code for analysis."""
    code_context = []
    
    for rs_file in repo_root.rglob("*.rs"):
        if "target" in str(rs_file) or "flycheck" in str(rs_file):
            continue
        
        rel_path = rs_file.relative_to(repo_root)
        content = rs_file.read_text()
        code_context.append(f"\n// ========== FILE: {rel_path} ==========\n{content}")
    
    return "\n".join(code_context)


def call_ai(prompt: str, config: dict) -> str:
    """Call the AI with the prompt."""
    if config.get("use_local"):
        client = OpenAI(
            api_key="ollama", 
            base_url=config.get("api_base", "http://localhost:11434/v1")
        )
    else:
        client = OpenAI(api_key=os.environ.get("OPENAI_API_KEY"))
    
    model = config["model"]
    print(f"🤖 Analyzing with {model}...")
    
    response = client.chat.completions.create(
        model=model,
        messages=[
            {"role": "system", "content": "You are a senior Rust architect analyzing code for refactoring opportunities."},
            {"role": "user", "content": prompt}
        ],
        temperature=0.3
    )
    
    return response.choices[0].message.content


def extract_json_from_response(response: str) -> list:
    """Extract JSON array from AI response."""
    # Try to find JSON in code blocks
    if "```json" in response:
        start = response.find("```json") + 7
        end = response.find("```", start)
        json_str = response[start:end].strip()
    elif "```" in response:
        start = response.find("```") + 3
        end = response.find("```", start)
        json_str = response[start:end].strip()
    else:
        # Try to parse the whole response
        json_str = response.strip()
    
    try:
        return json.loads(json_str)
    except json.JSONDecodeError as e:
        print(f"❌ Failed to parse JSON: {e}")
        print(f"Response preview: {response[:500]}...")
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
    
    # Call AI
    response = call_ai(full_prompt, config)
    
    # Parse response
    new_items = extract_json_from_response(response)
    
    if not new_items:
        print("⚠️ No refactoring opportunities found or failed to parse response")
        return 0
    
    # Generate unique IDs and add items
    for item in new_items:
        if "id" not in item or not item["id"]:
            item["id"] = generate_unique_id("refactoring", existing_ids + [i["id"] for i in new_items])
    
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
    repo_root = Path(__file__).parent.parent
    
    added = analyze(repo_root, config)
    
    print(f"\n{'=' * 60}")
    print(f"Analysis complete: {added} new items added")
    print(f"{'=' * 60}")


if __name__ == "__main__":
    main()
