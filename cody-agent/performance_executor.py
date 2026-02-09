"""
Performance Executor Agent

Executes a specific performance optimization from the TODO list.
"""

import os
import sys
import json
import subprocess
from pathlib import Path
from openai import OpenAI
from todo_manager import TodoList


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
    """Load the performance execution prompt."""
    repo_root = Path(__file__).parent.parent
    prompt_path = repo_root / ".github" / "ai" / "prompts" / "performance_execution.md"
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
            base_url=config.get("api_base", "http://localhost:11434/v1")
        )
    else:
        api_key = os.environ.get("OPENAI_API_KEY")
        if not api_key:
            print(f"\n❌ Error: OPENAI_API_KEY environment variable not set")
            print(f"\n   Set your API key:")
            print(f"   export OPENAI_API_KEY=sk-...")
            print(f"\n   Or configure 'use_local': true in config.json to use a local LLM.\n")
            sys.exit(1)
        client = OpenAI(api_key=api_key)
    
    model = config["model"]
    print(f"🤖 Implementing optimization with {model}...")
    
    response = client.chat.completions.create(
        model=model,
        messages=[
            {"role": "system", "content": "You are a performance optimization expert implementing optimizations."},
            {"role": "user", "content": prompt}
        ],
        temperature=0.2
    )
    
    return response.choices[0].message.content


def extract_patch(response: str) -> str:
    """Extract the patch from AI response."""
    if "```diff" in response:
        start = response.find("```diff") + 7
        end = response.find("```", start)
        return response[start:end].strip()
    elif "diff --git" in response:
        start = response.find("diff --git")
        return response[start:].strip()
    else:
        print("⚠️ No clear diff block found in response")
        return response


def apply_patch(repo_root: Path, patch: str) -> bool:
    """Apply the patch to the repository."""
    patch_file = repo_root / "temp_performance.patch"
    
    try:
        patch_file.write_text(patch)
        print(f"📄 Patch saved to: {patch_file}")
        
        result = subprocess.run(
            ["git", "apply", "--verbose", str(patch_file)],
            cwd=repo_root,
            capture_output=True,
            text=True
        )
        
        if result.returncode == 0:
            print("✅ Patch applied successfully")
            patch_file.unlink()
            return True
        else:
            print(f"❌ Failed to apply patch:")
            print(result.stderr)
            print(f"Patch saved at {patch_file} for manual review")
            return False
            
    except Exception as e:
        print(f"❌ Error applying patch: {e}")
        return False


def run_benchmarks(repo_root: Path) -> dict:
    """Run benchmarks and return results."""
    print("\n📊 Running benchmarks...")
    
    result = subprocess.run(
        ["cargo", "bench", "-p", "engine"],
        cwd=repo_root,
        capture_output=True,
        text=True,
        timeout=300
    )
    
    # Extract benchmark results (simplified)
    return {
        "success": result.returncode == 0,
        "output": result.stdout,
        "stderr": result.stderr
    }


def validate_changes(repo_root: Path) -> bool:
    """Run tests and benchmarks to validate the optimization."""
    print("\n🛡️ Validating optimization...")
    
    steps = [
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
            timeout=300
        )
        
        if result.returncode != 0:
            print(f"  ❌ {step_name} failed:")
            print(result.stderr)
            return False
        else:
            print(f"  ✅ {step_name} passed")
    
    return True


def execute_optimization(item_id: str, repo_root: Path, config: dict) -> bool:
    """Execute a specific performance optimization."""
    print("=" * 60)
    print(f"EXECUTING OPTIMIZATION: {item_id}")
    print("=" * 60)
    
    # Load TODO list
    todo_list = TodoList("performance", repo_root)
    
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
    
    # Run baseline benchmarks
    print("\n📊 Running baseline benchmarks...")
    baseline = run_benchmarks(repo_root)
    
    # Build the prompt
    prompt_template = get_prompt_template()
    
    performance_details = f"""
ID: {item.id}
Title: {item.title}
Priority: {item.priority}
Category: {item.category}
Current Bottleneck: {item.metadata.get('current_bottleneck', 'See description')}
Proposed Optimization: {item.metadata.get('proposed_optimization', 'See description')}
Expected Speedup: {item.metadata.get('expected_speedup', 'Unknown')}
Files Affected: {', '.join(item.files_affected)}
Requires Unsafe: {item.metadata.get('requires_unsafe', 'no')}

Description: {item.description}
"""
    
    # Gather relevant code
    code_context = ""
    if item.files_affected:
        code_context = gather_relevant_code(repo_root, item.files_affected)
    
    full_prompt = prompt_template.replace("{PERFORMANCE_DETAILS}", performance_details)
    full_prompt += f"\n\n## CODE CONTEXT\n\n{code_context}"
    
    # Call AI
    response = call_ai(full_prompt, config)
    
    # Extract and apply patch
    patch = extract_patch(response)
    
    if not patch:
        print("❌ Failed to extract patch from response")
        return False
    
    if not apply_patch(repo_root, patch):
        print("❌ Failed to apply patch")
        return False
    
    # Validate
    if not validate_changes(repo_root):
        print("❌ Validation failed, rolling back...")
        subprocess.run(["git", "checkout", "."], cwd=repo_root)
        return False
    
    # Run post-optimization benchmarks
    print("\n📊 Running post-optimization benchmarks...")
    optimized = run_benchmarks(repo_root)
    
    # Compare (simplified - in practice, parse and compare actual numbers)
    print("\n📈 Benchmark Comparison:")
    print("Baseline output:")
    print(baseline.get("output", "")[:500])
    print("\nOptimized output:")
    print(optimized.get("output", "")[:500])
    
    # Mark as completed
    todo_list.mark_completed(item_id)
    todo_list.save()
    
    print(f"\n✅ Optimization {item_id} completed successfully")
    return True


def main():
    """Main entry point."""
    import sys
    
    if len(sys.argv) < 2:
        print("Usage: python performance_executor.py <item_id>")
        print("   or: python performance_executor.py next")
        sys.exit(1)
    
    config = load_config()
    repo_root = Path(__file__).parent.parent
    
    item_id = sys.argv[1]
    
    if item_id == "next":
        todo_list = TodoList("performance", repo_root)
        next_item = todo_list.get_next_item()
        if not next_item:
            print("No items available to work on")
            sys.exit(0)
        item_id = next_item.id
        print(f"Working on next item: {item_id}")
    
    success = execute_optimization(item_id, repo_root, config)
    
    if success:
        print(f"\n{'=' * 60}")
        print("Optimization completed successfully")
        print(f"{'=' * 60}")
    else:
        print(f"\n{'=' * 60}")
        print("Optimization failed")
        print(f"{'=' * 60}")
        sys.exit(1)


if __name__ == "__main__":
    main()
