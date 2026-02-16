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
        return END
    return "rollback_changes"

# Initialize the Graph
builder = StateGraph(CodyState)

# Add Nodes
builder.add_node("clippy_agent", clippy_agent)
builder.add_node("apply_diff", apply_diff)
builder.add_node("run_clippy", run_clippy)
builder.add_node("run_build", run_build)
builder.add_node("run_tests", run_tests)
builder.add_node("rollback_changes", rollback_changes)

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

# 6. Tests -> End or Rollback
builder.add_conditional_edges(
    "run_tests",
    after_tests,
)

# 7. Rollback -> End
builder.add_edge("rollback_changes", END)

# Compile the graph into a runnable executable
app = builder.compile()