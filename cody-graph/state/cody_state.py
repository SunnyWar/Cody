from typing import TypedDict, List, Literal, Optional
from langgraph.graph import StateGraph, START, END

class CodyState(TypedDict):
    messages: List[dict]          # chat history
    repo_path: str                # path to Cody's repo
    last_command: Optional[str]   # e.g. "clippy"
    last_output: Optional[str]    # stdout/stderr from tools
    last_diff: Optional[str]      # last applied unified diff
    status: Literal["ok", "error", "pending"]
