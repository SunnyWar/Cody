# graph/cody_graph.py
from langgraph.graph import StateGraph, START, END
from state.cody_state import CodyState
from agents.clippy_agent import clippy_agent
from agents.elo_gain_agent import elo_gain_agent
from agents.ucifeatures_agent import ucifeatures_agent
from agents.performance_agent import performance_agent
from tools.run_build import run_build
from tools.run_clippy import run_clippy
from tools.run_tests import run_tests
from tools.run_fmt import run_fmt
from tools.apply_diff import apply_diff
from tools.rollback_changes import rollback_changes


def _retry_node_for_phase(state: CodyState) -> str:
    phase = state.get("current_phase", "clippy")
    if phase == "clippy":
        return "run_clippy"
    elif phase == "performance":
        return "performance_agent"
    else:
        return "clippy_agent"

def route_phase(state: CodyState) -> str:
    """
    Route to the appropriate phase agent based on current_phase.
    
    Different phases have fundamentally different workflows:
    - Clippy: Detect and fix pedantic clippy warnings (run_clippy)
    - Refactoring, Tests: Use clippy_agent (iterative LLM fixes, no clippy checks)
    - Performance: Use performance_agent (targeted performance optimization strategies)
    - UCIfeatures: Use ucifeatures_agent (UCI protocol expansion)
    - ELOGain: Use elo_gain_agent (complex multi-step chess improvement loop)
    """
    phase = state.get("current_phase", "clippy")
    
    if phase == "ELOGain":
        return "elo_gain_agent"
    elif phase == "UCIfeatures":
        return "ucifeatures_agent"
    elif phase == "performance":
        return "performance_agent"
    elif phase == "clippy":
        # Clippy phase: run clippy to detect warnings
        return "run_clippy"
    else:
        # All other phases (refactoring, tests, etc.): skip clippy, go directly to LLM agent
        return "clippy_agent"

def after_elo_gain(state: CodyState) -> str:
    """Route after ELO Gain phase completes."""
    # ELO Gain agent manages its own state machine; when done, move to next phase
    return "phase_complete"

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
        if state.get("current_phase") in ("UCIfeatures", "performance"):
            return "phase_complete"
        return END
    if state["status"] == "ok":
        # Refactoring may complete with "no beneficial strategy found" and no patch.
        if state.get("current_phase") == "refactoring" and state.get("last_command") == "clippy_llm_think":
            return "phase_complete"
        if state.get("current_phase") in ("UCIfeatures", "performance"):
            # Performance-only: When agent completes, proceed to build/test iteration
            return "run_build"
        # All warnings attempted, proceed to build
        return "run_build"
    return "apply_diff"

def after_apply_diff(state: CodyState):
    if state["status"] == "error":
        return END
    # If diff was rejected or no diff was generated, retry according to phase workflow.
    if state.get("last_command") in ("apply_diff_rejected", "apply_diff_no_diff"):
        if state.get("current_phase") in ("UCIfeatures", "performance"):
            phase_name = state.get("current_phase", "unknown")
            print(f"[cody-graph] [DIAG] {phase_name}: no patch produced/applied; ending phase", flush=True)
            return "phase_complete"
        retry_node = _retry_node_for_phase(state)
        print(f"[cody-graph] [DIAG] Diff unavailable/rejected; retrying via {retry_node}", flush=True)
        return retry_node
    # After applying a patch, immediately validate that it compiles
    print("[cody-graph] [DIAG] Patch applied, validating with build", flush=True)
    return "run_build"

def after_build(state: CodyState):
    if state["status"] == "ok":
        return "run_tests"

    if state.get("current_phase") in ("UCIfeatures", "performance") and state.get("last_diff"):
        phase_name = state.get("current_phase", "unknown")
        print(f"[cody-graph] [DIAG] {phase_name}: build failed after single change; rolling back and ending phase", flush=True)
        return "rollback_changes"
    
    # Build failed - determine context for routing
    # If we were validating a patch post-apply_diff, check if we should attempt repair
    if state.get("last_diff"):
        # Check if build failed after a patch and we should attempt repair
        from tools.retry_manager import create_retry_manager
        retry_mgr = create_retry_manager(state)
        
        if retry_mgr.should_attempt_repair(state):
            print(
                "[cody-graph] [DIAG] Build failed after patch; attempting LLM repair",
                flush=True,
            )
            return "build_repair_attempt"
        else:
            print("[cody-graph] [DIAG] Build failed; repair attempts exhausted, rolling back", flush=True)
            return "rollback_changes"
    
    # Otherwise (if validating after clippy phase), any build failure is critical
    return "rollback_changes"

def after_build_repair_attempt(state: CodyState):
    """After LLM generates a repair, apply it and try building again."""
    if state["status"] == "error":
        return END
    if state["status"] == "ok":
        # All warnings attempted, move on
        return "run_build"
    # Apply the repair patch
    return "apply_diff"

def after_rollback(state: CodyState):
    """Route after rollback: handle failed repairs and continue with next warning."""
    from tools.retry_manager import create_retry_manager
    retry_mgr = create_retry_manager(state)
    
    if state.get("current_phase") in ("UCIfeatures", "performance"):
        phase_name = state.get("current_phase", "unknown")
        print(f"[cody-graph] [DIAG] {phase_name}: rollback complete, ending phase", flush=True)
        return "phase_complete"

    # If rollback happened after a failed repair attempt:
    # 1. Mark the warning as permanently failed
    # 2. Reset repair counter for next attempt
    # 3. Go back to clippy to try another warning
    if state.get("last_command") == "cargo_build" and state.get("last_diff"):
        phase = state.get("current_phase", "clippy")
        warning_sig = state.get("current_warning_signature")
        repair_attempts = int(state.get("repair_attempts", 0) or 0)
        
        if warning_sig:
            retry_mgr.mark_warning_failed(
                phase,
                warning_sig,
                f"Build failed after {repair_attempts} repair attempt(s)",
                repair_attempts,
            )
            # Mark attempted so clippy agent won't try it again
            attempted = state.get("attempted_warnings", []) or []
            if warning_sig not in attempted:
                attempted = attempted + [warning_sig]
        
        retry_node = _retry_node_for_phase(state)
        print(
            "[cody-graph] [DIAG] Rollback after repair failure; "
            f"retrying via {retry_node}",
            flush=True,
        )
        return retry_node
    
    # If rollback happened after apply_diff (malformed patch, etc.):
    # Mark the warning as attempted but keep retrying
    if state.get("last_command") == "apply_diff":
        retry_node = _retry_node_for_phase(state)
        print(f"[cody-graph] [DIAG] Rollback after failed patch application, retrying via {retry_node}", flush=True)
        return retry_node
    
    # Otherwise, end the phase (critical build failure during validation)
    return END

def after_tests(state: CodyState):
    if state["status"] == "ok":
        return "run_fmt"

    if state.get("current_phase") in ("UCIfeatures", "performance") and state.get("last_diff"):
        phase_name = state.get("current_phase", "unknown")
        print(f"[cody-graph] [DIAG] {phase_name}: tests failed after single change; rolling back and ending phase", flush=True)
        return "rollback_changes"

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
            "repair_attempts": 0,  # Reset repair attempts for new phase
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
        # More phases to do - route through phase router for next phase
        return "route_phase"
    return END

# Initialize the Graph
builder = StateGraph(CodyState)

# Add Nodes
builder.add_node("route_phase", lambda state: {"current_phase": state.get("current_phase", "clippy")})  # Routing node (no-op, just decides direction)
builder.add_node("clippy_agent", clippy_agent)
builder.add_node("elo_gain_agent", elo_gain_agent)
builder.add_node("ucifeatures_agent", ucifeatures_agent)
builder.add_node("performance_agent", performance_agent)
builder.add_node("apply_diff", apply_diff)
builder.add_node("run_clippy", run_clippy)
builder.add_node("run_build", run_build)
builder.add_node("run_tests", run_tests)
builder.add_node("run_fmt", run_fmt)
builder.add_node("rollback_changes", rollback_changes)
builder.add_node("phase_complete", phase_complete)

# Build Repair Node: Increment repair counter and call clippy_agent to fix build errors
def build_repair_attempt(state: CodyState) -> CodyState:
    """Attempt to repair a build failure with LLM."""
    from tools.retry_manager import create_retry_manager
    retry_mgr = create_retry_manager(state)
    
    # Increment repair attempt counter
    state = retry_mgr.increment_repair_attempts(state)
    
    # Mark this as a repair attempt so clippy_agent knows context
    repair_attempt_num = int(state.get("repair_attempts", 1) or 1)
    print(
        f"[cody-graph] [DIAG] Build Repair Attempt #{repair_attempt_num}",
        flush=True,
    )
    
    # Call clippy_agent with build error context - it will generate a repair
    return clippy_agent(state)

builder.add_node("build_repair_attempt", build_repair_attempt)

# Define Flow
builder.add_edge(START, "route_phase")

# Route to appropriate phase agent
builder.add_conditional_edges(
    "route_phase",
    route_phase,
)

# ELO Gain Agent -> Phase Complete (when internal state machine finishes)
builder.add_edge("elo_gain_agent", "phase_complete")

# 2. Agent -> Apply Diff (Write fix to disk)
builder.add_conditional_edges(
    "clippy_agent",
    after_clippy_agent,
)

builder.add_conditional_edges(
    "performance_agent",
    after_clippy_agent,
)

builder.add_conditional_edges(
    "ucifeatures_agent",
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

# 5. Build -> Tests, Repair Attempt, or Rollback
builder.add_conditional_edges(
    "run_build",
    after_build,
)

# 5b. Build Repair Attempt -> Apply Diff or End
builder.add_conditional_edges(
    "build_repair_attempt",
    after_build_repair_attempt,
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