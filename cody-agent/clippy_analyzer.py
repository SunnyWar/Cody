"""
Clippy Analysis Agent

Runs cargo clippy and turns warnings into actionable TODO items.
"""

import os
import sys
import json
import subprocess
from pathlib import Path
from openai import OpenAI
from todo_manager import TodoList, generate_unique_id


CLIPPY_LINT_ARGS = [
    "-W", "clippy::perf",
    "-W", "clippy::inline_always",
    "-W", "clippy::large_stack_arrays",
    "-W", "clippy::large_types_passed_by_value",
    "-W", "clippy::large_enum_variant",
    "-W", "clippy::needless_pass_by_ref_mut",
    "-W", "clippy::box_collection",
    "-W", "clippy::vec_box",
    "-W", "clippy::rc_buffer",
    "-W", "clippy::pedantic",
    "-W", "clippy::undocumented_unsafe_blocks",
]

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
    """Load the clippy analysis prompt."""
    repo_root = Path(__file__).parent.parent
    prompt_path = repo_root / ".github" / "ai" / "prompts" / "clippy_analysis.md"
    return prompt_path.read_text()


def run_clippy(repo_root: Path) -> dict:
    """Run cargo clippy and return results."""
    command = [
        "cargo",
        "clippy",
        "--all-targets",
        "--all-features",
        "--",
        *CLIPPY_LINT_ARGS,
    ]

    result = subprocess.run(
        command,
        cwd=repo_root,
        capture_output=True,
        text=True
    )

    output = "\n".join([
        "# --- STDOUT ---",
        result.stdout.strip(),
        "# --- STDERR ---",
        result.stderr.strip()
    ]).strip()

    return {
        "command": " ".join(command),
        "returncode": result.returncode,
        "output": output
    }


def trim_output(output: str, limit: int = 12000) -> str:
    """Trim long outputs to a reasonable size."""
    if len(output) <= limit:
        return output

    head = output[:6000]
    tail = output[-4000:]
    return f"{head}\n\n... [truncated] ...\n\n{tail}"


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
    print(f"🤖 Analyzing clippy with {model}...")

    response = client.chat.completions.create(
        model=model,
        messages=[
            {
                "role": "system",
                "content": "You are a senior Rust engineer analyzing clippy output."
            },
            {"role": "user", "content": prompt}
        ],
        temperature=0.2
    )

    return response.choices[0].message.content


def extract_json_from_response(response: str) -> list:
    """Extract JSON array from AI response."""
    if "```json" in response:
        start = response.find("```json") + 7
        end = response.find("```", start)
        json_str = response[start:end].strip()
    elif "```" in response:
        start = response.find("```") + 3
        end = response.find("```", start)
        json_str = response[start:end].strip()
    else:
        json_str = response.strip()

    try:
        return json.loads(json_str)
    except json.JSONDecodeError as e:
        print(f"❌ Failed to parse JSON: {e}")
        print(f"Response preview: {response[:500]}...")
        return []


def analyze(repo_root: Path, config: dict) -> int:
    """Run clippy analysis and update TODO list."""
    print("=" * 60)
    print("CLIPPY ANALYSIS")
    print("=" * 60)

    todo_list = TodoList("clippy", repo_root)
    existing_ids = todo_list.get_all_ids()

    prompt_template = get_prompt_template()

    clippy_result = run_clippy(repo_root)
    clippy_output = trim_output(clippy_result["output"])

    existing_todos_info = ""
    if todo_list.items:
        existing_todos_info = "\n## Existing TODO Items (DO NOT DUPLICATE)\n\n"
        for item in todo_list.items:
            existing_todos_info += f"- {item.id}: {item.title} [{item.status}]\n"

    clippy_status = (
        "Clippy completed successfully."
        if clippy_result["returncode"] == 0
        else f"Clippy exited with code {clippy_result['returncode']}."
    )

    full_prompt = (
        f"{prompt_template}\n\n"
        f"{existing_todos_info}\n\n"
        f"## CLIPPY COMMAND\n\n{clippy_result['command']}\n\n"
        f"## CLIPPY STATUS\n\n{clippy_status}\n\n"
        f"## CLIPPY OUTPUT\n\n{clippy_output}"
    )

    response = call_ai(full_prompt, config)

    new_items = extract_json_from_response(response)
    
    # Validate that new_items is a list of dicts
    if not isinstance(new_items, list):
        print(f"⚠️ Expected list of items, got {type(new_items).__name__}")
        return 0
    
    # Filter out non-dict items
    new_items = [item for item in new_items if isinstance(item, dict)]
    if not new_items:
        print("⚠️ No clippy opportunities found or failed to parse response")
        return 0

    for item in new_items:
        if "id" not in item or not item["id"]:
            existing_new_ids = [i.get("id") for i in new_items if isinstance(i, dict) and i.get("id")]
            item["id"] = generate_unique_id("clippy", existing_ids + existing_new_ids)

    added = todo_list.add_items(new_items, check_duplicates=True)

    if added > 0:
        todo_list.save()
        print(f"\n✅ Added {added} new clippy items to TODO list")
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
