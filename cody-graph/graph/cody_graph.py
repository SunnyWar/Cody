# graph/cody_graph.py
from langgraph.graph import StateGraph, START, END
from state.cody_state import CodyState
from agents.clippy_agent import clippy_agent
from tools.run_build import run_build
from tools.run_clippy import run_clippy
from tools.run_tests import run_tests
from tools.apply_diff import apply_diff
from tools.rollback_changes import rollback_changes

def after_clippy(state: CodyState):
    if state["status"] == "ok":
        return "run_build"
    return "clippy_agent"

def after_clippy_agent(state: CodyState):
    if state["status"] == "error":
        return END
    return "apply_diff"

def after_apply_diff(state: CodyState):
    if state["status"] == "error":
        return END
    return "run_clippy"

def after_build(state: CodyState):
    if state["status"] == "ok":
        return "run_tests"
    return "rollback_changes"

def after_tests(state: CodyState):
    if state["status"] == "ok":
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

# 6. Tests -> Phase Complete or Rollback
builder.add_conditional_edges(
    "run_tests",
    after_tests,
)

# 7. Phase Complete -> Next Phase or End
builder.add_conditional_edges(
    "phase_complete",
    after_phase_complete,
)

# 8. Rollback -> End
builder.add_edge("rollback_changes", END)

# Compile the graph into a runnable executable
app = builder.compile()