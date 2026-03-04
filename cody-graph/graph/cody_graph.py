# graph/cody_graph.py
from langgraph.graph import StateGraph, START, END
from state.cody_state import CodyState
from agents.clippy_agent import clippy_agent
from tools.run_build import run_build
from tools.run_clippy import run_clippy
from tools.run_tests import run_tests
from tools.run_fmt import run_fmt
from tools.apply_diff import apply_diff
from tools.rollback_changes import rollback_changes

def after_clippy(state: CodyState):
    if state["status"] == "ok":
        return "run_build"

    # CRITICAL: Syntax/parse errors must rollback immediately
    if state.get("clippy_has_syntax_error") and state.get("last_diff"):
        print(
            "[cody-graph] [DIAG] Critical syntax error introduced by last patch; routing to rollback",
            flush=True,
        )
        return "rollback_changes"

    # Safety gate: if an applied diff worsens clippy errors, rollback immediately.
    current_errors = state.get("clippy_error_count")
    best_errors = state.get("best_clippy_error_count")
    if (
        isinstance(current_errors, int)
        and isinstance(best_errors, int)
        and current_errors > best_errors
        and state.get("last_diff")
    ):
        print(
            "[cody-graph] [DIAG] Clippy regression detected "
            f"(current={current_errors}, best={best_errors}); routing to rollback",
            flush=True,
        )
        return "rollback_changes"

    return "clippy_agent"

def after_clippy_agent(state: CodyState):
    if state["status"] == "error":
        return END
    if state["status"] == "ok":
        # All warnings attempted, proceed to build
        return "run_build"
    return "apply_diff"

def after_apply_diff(state: CodyState):
    if state["status"] == "error":
        return END
    # If diff was rejected (too many lines, etc.), loop back to clippy to try next warning
    if state.get("last_command") == "apply_diff_rejected":
        print("[cody-graph] [DIAG] Diff rejected, running clippy again for next warning", flush=True)
        return "run_clippy"
    # After applying a patch, immediately validate that it compiles
    print("[cody-graph] [DIAG] Patch applied, validating with build", flush=True)
    return "run_build"

def after_build(state: CodyState):
    if state["status"] == "ok":
        return "run_tests"
    # Build failed - determine context for routing
    # If we were validating a patch post-apply_diff, rollback and retry clippy
    if state.get("last_command") == "apply_diff":
        print("[cody-graph] [DIAG] Build failed after patch application, rolling back", flush=True)
        return "rollback_changes"
    # Otherwise (if validating after clippy phase), any build failure is critical
    return "rollback_changes"

def after_rollback(state: CodyState):
    """Route after rollback: either retry clippy or end phase."""
    # If rollback happened after a failed patch validation, retry clippy
    if state.get("last_command") == "apply_diff":
        print("[cody-graph] [DIAG] Rollback complete, retrying clippy for next warning", flush=True)
        return "run_clippy"
    # Otherwise, end the phase (critical build failure)
    return END

def after_tests(state: CodyState):
    if state["status"] == "ok":
        return "run_fmt"

    # For test failures after a patch: give AI one repair attempt before rollback.
    if state.get("last_diff"):
        failures = int(state.get("consecutive_test_failures", 0) or 0)
        if failures <= 1:
            print(
                "[cody-graph] [DIAG] Test failure after patch; routing to AI repair attempt",
                flush=True,
            )
            return "clippy_agent"
        print(
            "[cody-graph] [DIAG] Test repair attempt failed; rolling back patch",
            flush=True,
        )
        return "rollback_changes"

    # If tests fail with no known patch context, rollback/end as before.
    return "rollback_changes"

def after_fmt(state: CodyState):
    if state["status"] == "ok":
        print("[cody-graph] [DIAG] Code formatted, phase complete", flush=True)
        return "phase_complete"
    return "rollback_changes"

def phase_complete(state: CodyState) -> CodyState:
    """Transition to the next phase or end orchestration."""
    print("[cody-graph] phase_complete: START", flush=True)
    
    current = state["current_phase"]
    completed = state["phases_completed"] + [current]
    todo = state["phases_todo"]
    
    print(f"[cody-graph] [DIAG] Phase '{current}' completed", flush=True)
    print(f"[cody-graph] [DIAG] Completed phases: {completed}", flush=True)
    print(f"[cody-graph] [DIAG] Remaining phases: {todo}", flush=True)
    
    if todo:
        next_phase = todo[0]
        result = {
            **state,
            "current_phase": next_phase,
            "phases_todo": todo[1:],
            "phases_completed": completed,
            "phase_iteration": 0,
            "consecutive_test_failures": 0,
            "clippy_error_count": None,
            "best_clippy_error_count": None,
            "clippy_has_syntax_error": None,
            "attempted_warnings": [],  # Reset for new phase
            "last_output": f"Phase '{current}' complete. Starting phase '{next_phase}'.",
            "status": "pending",
        }
        print(f"[cody-graph] [DIAG] Transitioning to phase: {next_phase}", flush=True)
        print("[cody-graph] phase_complete: END (next phase)", flush=True)
        return result
    else:
        result = {
            **state,
            "phases_completed": completed,
            "last_output": f"All phases complete: {completed}",
            "status": "ok",
        }
        print(f"[cody-graph] [DIAG] All phases completed!", flush=True)
        print("[cody-graph] phase_complete: END (all done)", flush=True)
        return result

def after_phase_complete(state: CodyState):
    """Route after phase completion: either to next phase or to END."""
    if state["phases_todo"]:
        # More phases to do - run the analysis/clippy for next phase
        # For now, we restart at run_clippy; could add phase-specific agents later
        return "run_clippy"
    return END

# Initialize the Graph
builder = StateGraph(CodyState)

# Add Nodes
builder.add_node("clippy_agent", clippy_agent)
builder.add_node("apply_diff", apply_diff)
builder.add_node("run_clippy", run_clippy)
builder.add_node("run_build", run_build)
builder.add_node("run_tests", run_tests)
builder.add_node("run_fmt", run_fmt)
builder.add_node("rollback_changes", rollback_changes)
builder.add_node("phase_complete", phase_complete)

# Define Flow
builder.add_edge(START, "run_clippy")

# 2. Agent -> Apply Diff (Write fix to disk)
builder.add_conditional_edges(
    "clippy_agent",
    after_clippy_agent,
)

# 3. Apply Diff -> Run Clippy (Verify fix)
builder.add_conditional_edges(
    "apply_diff",
    after_apply_diff,
)

# 4. Run Clippy -> Build or Loop
builder.add_conditional_edges(
    "run_clippy",
    after_clippy,
)

# 5. Build -> Tests or Rollback
builder.add_conditional_edges(
    "run_build",
    after_build,
)

# 6. Tests -> Format or Rollback
builder.add_conditional_edges(
    "run_tests",
    after_tests,
)

# 6b. Format -> Check for more warnings or Rollback
builder.add_conditional_edges(
    "run_fmt",
    after_fmt,
)

# 7. Phase Complete -> Next Phase or End
builder.add_conditional_edges(
    "phase_complete",
    after_phase_complete,
)

# 8. Rollback -> Next Step (retry clippy or end)
builder.add_conditional_edges(
    "rollback_changes",
    after_rollback,
)

# Compile the graph into a runnable executable
app = builder.compile()