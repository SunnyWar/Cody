"""
Clippy Analysis Agent

Runs cargo clippy and turns warnings into actionable TODO items.
"""

import os
import sys
import json
import re
from datetime import datetime
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
    return prompt_path.read_text(encoding='utf-8')


def run_clippy_with_parser(repo_root: Path, extra_args: list = None) -> dict:
    """Run clippy_parser.py to execute Clippy and collect warnings."""
    parser_script = repo_root / "cody-agent" / "clippy_parser.py"
    command = ["python", str(parser_script)]

    if extra_args:
        command.append("--")
        command.extend(extra_args)

    print(f"\nRunning Clippy parser command: {' '.join(command)}")

    result = subprocess.run(
        command,
        cwd=repo_root,
        capture_output=True,
        text=True
    )

    if result.returncode not in (0, 1):
        print(f"\n⚠️ Clippy parser exited with code {result.returncode}")

    warnings = []
    for line in result.stdout.strip().split('\n'):
        if not line.strip():
            continue
        try:
            warnings.append(json.loads(line))
        except json.JSONDecodeError:
            print(f"⚠️ Failed to parse line as JSON: {line[:200]}...")

    return {
        "command": " ".join(command),
        "returncode": result.returncode,
        "warnings": warnings
    }


def run_clippy_with_priority_and_parser(repo_root: Path) -> dict:
    """Run clippy_parser.py with prioritized lint options, stopping when warnings are found."""
    prioritized_lints = [
        [],
        ["-W", "clippy::inline_always"],
        ["-W", "clippy::large_stack_arrays"],
        ["-W", "clippy::large_types_passed_by_value"],
        ["-W", "clippy::large_enum_variant"],
        ["-W", "clippy::needless_pass_by_ref_mut"],
        ["-W", "clippy::box_collection"],
        ["-W", "clippy::vec_box"],
        ["-W", "clippy::rc_buffer"],
        ["-W", "clippy::undocumented_unsafe_blocks"],
        ["-W", "clippy::pedantic"],
        ["-W", "clippy::perf"],
        ["-W", "clippy::style"],
        ["-W", "clippy::correctness"],
    ]

    all_warnings = []

    for lints in prioritized_lints:
        parser_script = repo_root / "cody-agent" / "clippy_parser.py"
        command = ["python", str(parser_script)]
        if lints:
            command += ["--"] + lints

        print(f"\nRunning Clippy parser command: {' '.join(command)}")

        result = subprocess.run(
            command,
            cwd=repo_root,
            capture_output=True,
            text=True
        )

        # Parse stdout for warnings
        warnings = []
        for line in result.stdout.strip().split("\n"):
            if line.strip():
                try:
                    warnings.append(json.loads(line))
                except json.JSONDecodeError:
                    print(f"⚠️ Failed to parse line as JSON: {line[:200]}...")

        # Add warnings to the cumulative list
        all_warnings.extend(warnings)

        # Check exit code to determine if warnings exist
        lint_label = "(default)" if not lints else " ".join(lints)
        if result.returncode == 1:  # Warnings found
            print(f"\n📊 Found {len(warnings)} warnings with lints: {lint_label}")
        elif result.returncode == 0:  # No warnings
            print(f"\n📊 Found 0 warnings with lints: {lint_label}. Continuing to next lint option.")

    if all_warnings:
        warning_summary = "\n".join(
            [f"{i+1}. {warning['message']['code']['code']}: {warning['message']['spans'][0]['file_name']}" for i, warning in enumerate(all_warnings)]
        )
        return {
            "command": "Multiple Clippy Commands",
            "returncode": 1,
            "warnings": all_warnings,
            "output": warning_summary,
            "warning_count": len(all_warnings),
        }

    print("\n✅ No warnings found for any Clippy lint options.")
    return {
        "command": "",
        "returncode": 0,
        "warnings": [],
        "output": "No warnings found.",
        "warning_count": 0,
    }


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
    print(f"🤖 Analyzing clippy with {model}...")

    response = client.chat.completions.create(
        model=model,
        messages=[
            {
                "role": "system",
                "content": "You are a senior Rust engineer analyzing clippy output. Your task is to provide actionable code modifications to fix the issues identified by Clippy. Respond with the modified code directly, and include comments to explain the changes where necessary."
            },
            {
                "role": "user",
                "content": prompt
            }
        ],
        temperature=0.2
    )

    return response.choices[0].message.content  # Return the raw response, which may include code


def extract_json_from_response(response: str, repo_root: Path, phase: str) -> list:
    """Extract JSON array from AI response using multiple strategies."""
    json_str = None
    
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
        # Ensure we return a list
        return result if isinstance(result, list) else []
    except json.JSONDecodeError as e:
        logs_dir = repo_root / ".orchestrator_logs"
        logs_dir.mkdir(parents=True, exist_ok=True)
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


def extract_code_from_response(response: str, repo_root: Path, phase: str) -> list:
    """Extract code modifications from the LLM response."""
    if not response.strip():
        print("⚠️ Empty response from LLM.")
        return []

    # Save the response to a temporary file for review
    logs_dir = repo_root / ".orchestrator_logs"
    logs_dir.mkdir(parents=True, exist_ok=True)
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    response_path = logs_dir / f"clippy_code_response_{timestamp}.txt"
    with response_path.open("w", encoding="utf-8", errors="replace") as f:
        f.write(response)

    print(f"📄 LLM response saved to: {response_path}")

    # Return the raw response for now (future: parse and apply changes automatically)
    return [response]


def analyze(repo_root: Path, config: dict) -> int:
    """Run clippy analysis and update TODO list."""
    print("=" * 60)
    print("CLIPPY ANALYSIS")
    print("=" * 60)

    todo_list = TodoList("clippy", repo_root)
    existing_ids = todo_list.get_all_ids()

    clippy_result = run_clippy_with_priority_and_parser(repo_root)
    clippy_output = clippy_result["output"]

    # Show warning summary
    warning_count = clippy_result["warning_count"]
    print(f"\n📊 Found {warning_count} clippy warnings")

    if warning_count == 0:
        print("\n✅ No warnings to process.")
        return 0

    new_items = []
    for warning in clippy_result["warnings"]:
        message = warning.get("message", {})
        spans = message.get("spans", [])
        span = spans[0] if spans else {}

        file_name = span.get("file_name", "")
        line_start = span.get("line_start", 0)
        column_start = span.get("column_start", 0)
        lint_code = message.get("code", {}).get("code", "clippy")
        rendered = message.get("rendered", message.get("message", ""))

        title = f"{lint_code}: {Path(file_name).name}" if file_name else lint_code

        new_items.append({
            "id": "",
            "title": title,
            "priority": "medium",
            "category": "clippy",
            "description": rendered,
            "status": "not-started",
            "estimated_complexity": "small",
            "files_affected": [file_name] if file_name else [],
            "dependencies": [],
            "file": file_name,
            "line": line_start,
            "column": column_start,
            "lint_name": lint_code,
            "lint_message": message.get("message", ""),
            "rendered": rendered,
        })

    if not new_items:
        print("⚠️ No clippy opportunities found")
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
