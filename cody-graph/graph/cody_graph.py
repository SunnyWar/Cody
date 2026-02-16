# graph/cody_graph.py
from langgraph.graph import StateGraph, START, END
from state.cody_state import CodyState
from agents.clippy_agent import clippy_agent
from tools.run_clippy import run_clippy
from tools.apply_diff import apply_diff

def should_continue(state: CodyState):
    """
    Determines if we should loop back to the agent or finish.
    We finish if status is 'ok' (no warnings) or if we hit an unrecoverable error.
    """
    if state["status"] == "ok":
        return END
    return "clippy_agent"

# Initialize the Graph
builder = StateGraph(CodyState)

# Add Nodes
builder.add_node("read_repo", read_repo)
builder.add_node("clippy_agent", clippy_agent)
builder.add_node("apply_diff", apply_diff)
builder.add_node("run_clippy", run_clippy)

# Define Flow
builder.add_edge(START, "read_repo")

# 1. Start -> Agent (Think of a fix)
builder.add_edge("read_repo", "clippy_agent")

# 2. Agent -> Apply Diff (Write fix to disk)
builder.add_edge("clippy_agent", "apply_diff")

# 3. Apply Diff -> Run Clippy (Verify fix)
builder.add_edge("apply_diff", "run_clippy")

# 4. Run Clippy -> Check Status (Loop or End)
builder.add_conditional_edges(
    "run_clippy",
    should_continue,
)

# Compile the graph into a runnable executable
app = builder.compile()