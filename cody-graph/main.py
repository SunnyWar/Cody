# main.py
import os
import json
import sys
from pathlib import Path

from graph.cody_graph import app
from state.cody_state import CodyState
from tools.phase_manager import save_phase_state

repo_root = Path(os.environ.get("CODY_REPO_PATH", Path(__file__).resolve().parents[1]))

PHASE_CLI_ALIASES = {
    "unit_tests_docs": "tests",
    "ELOGain": "elogain",
    "UCIfeatures": "ucifeatures",
}


def _to_cli_phase(phase: str) -> str:
    return PHASE_CLI_ALIASES.get(phase, phase)


def _to_internal_phase(selection: str, phases: list[str]) -> str | None:
    if selection in phases:
        return selection
    for internal, cli in PHASE_CLI_ALIASES.items():
        if selection == cli and internal in phases:
            return internal
    return None

def _load_phases_config(repo_root: Path) -> list:
    """Load and order phases configuration from cody-agent/config.json.
    
    IMPORTANT: Clippy MUST be first to fix compilation errors before other phases.
    """
    config_path = repo_root / "cody-agent" / "config.json"
    phases = []
    
    if config_path.exists():
        try:
            config = json.loads(config_path.read_text())
            # Extract model assignments as phases
            models = config.get("models", {})
            phases = list(models.keys())
            print(f"[cody-graph] [DIAG] Loaded phases from config: {phases}", flush=True)
        except Exception as e:
            print(f"[cody-graph] [DIAG] Failed to load phases config: {e}", flush=True)
    
    # Ensure clippy is ALWAYS first (to fix compilation errors before refactoring)
    if "clippy" in phases:
        phases.remove("clippy")
    phases.insert(0, "clippy")
    
    print(f"[cody-graph] [DIAG] Final phase order (clippy first): {phases}", flush=True)
    return phases


def _print_usage(phases: list[str]) -> None:
    print("Usage:")
    print("  python .\\cody-graph\\main.py <phase|all>")
    print("")
    print("Options:")
    print("  all          Run full orchestration (current behavior)")
    for phase in phases:
        cli_phase = _to_cli_phase(phase)
        print(f"  {cli_phase:<12} Run only the '{cli_phase}' phase")
    print("")
    print("Examples:")
    print("  python .\\cody-graph\\main.py all")
    print("  python .\\cody-graph\\main.py clippy")
    print("  python .\\cody-graph\\main.py refactoring")
    print("  python .\\cody-graph\\main.py tests")

phases_list = _load_phases_config(repo_root)

if len(sys.argv) < 2:
    _print_usage(phases_list)
    raise SystemExit(0)

selection = sys.argv[1].strip().lower()

if selection == "all":
    scheduled_phases = phases_list
else:
    selected_phase = _to_internal_phase(selection, phases_list)
    if selected_phase is not None:
        scheduled_phases = [selected_phase]
    else:
        print(f"Invalid phase option: {selection}")
        print("")
        _print_usage(phases_list)
        raise SystemExit(1)

if selection != "all" and not scheduled_phases:
    print(f"Invalid phase option: {selection}")
    print("")
    _print_usage(phases_list)
    raise SystemExit(1)

first_phase = scheduled_phases[0] if scheduled_phases else "clippy"
remaining_phases = scheduled_phases[1:] if len(scheduled_phases) > 1 else []

initial_state: CodyState = {
    "messages": [
        {
            "role": "user",
            "content": f"Please improve the Cody chess engine. Starting with the '{first_phase}' phase.",
        }
    ],
    "repo_path": str(repo_root),
    "last_command": None,
    "last_output": None,
    "status": "pending",
    "llm_response": None,
    "diff_extracted": None,
    "logs_dir": None,
    "changed_files": [],
    "consecutive_test_failures": 0,
    "clippy_error_count": None,
    "best_clippy_error_count": None,
    "clippy_has_syntax_error": None,
    "current_phase": first_phase,
    "phases_todo": remaining_phases,
    "phases_completed": [],
    "phase_iteration": 0,
    "attempted_warnings": [],
    "current_warning_signature": None,
}

print("=" * 80)
print("CODY-GRAPH: Multi-Phase Automated Improvement Agent")
print("=" * 80)
print(f"Repository: {repo_root}")
print(f"Phases scheduled: {[_to_cli_phase(phase) for phase in scheduled_phases]}")
print("=" * 80)

result = app.invoke(initial_state)

# Save phase state for future reference/resumption
save_phase_state(str(repo_root), result)

status = result["status"]
last_command = result.get("last_command")
logs_dir = result.get("logs_dir")
phases_completed = result.get("phases_completed", [])

if status == "ok":
    summary = f"Success: All phases completed {phases_completed}"
elif last_command == "rollback":
    summary = "Rollback applied after validation failure."
else:
    summary = f"Stopped with errors (phase: {result.get('current_phase')})."

print("\n" + "=" * 80)
print("FINAL RESULT")
print("=" * 80)
print(f"STATUS: {status}")
print(f"CURRENT_PHASE: {result.get('current_phase')}")
print(f"PHASES_COMPLETED: {phases_completed}")
print(f"SUMMARY: {summary}")

if logs_dir:
    print(f"\nDiagnostic logs saved to: {logs_dir}")
    log_files = sorted(Path(logs_dir).glob("*.log"))
    if log_files:
        print(f"Found {len(log_files)} diagnostic files:")
        for f in log_files[-5:]:  # Show last 5 logs
            print(f"  - {f.name}")

print("\n--- LLM MESSAGES ---")
for i, m in enumerate(result["messages"]):
    role = m["role"].upper()
    content = m["content"]
    preview = content[:300] if content else "(empty)"
    print(f"\n[{i}] {role}:")
    print(preview + ("..." if len(content) > 300 else ""))

print("\n--- LAST OUTPUT ---")
last_output = result.get("last_output", "")
if last_output:
    print(last_output[:1000])
else:
    print("(no output)")

print("\n" + "=" * 80)
