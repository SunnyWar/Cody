from typing import TypedDict, List, Literal, Optional
from langgraph.graph import StateGraph, START, END

class CodyState(TypedDict):
    messages: List[dict]          # chat history
    repo_path: str                # path to Cody's repo
    last_command: Optional[str]   # e.g. "clippy"
    last_output: Optional[str]    # stdout/stderr from tools
    last_diff: Optional[str]      # last applied unified diff
    status: Literal["ok", "error", "pending"]
    llm_response: Optional[str]   # raw LLM response for debugging
    diff_extracted: Optional[str] # extracted diff for debugging
    logs_dir: Optional[str]       # directory where logs are saved
    changed_files: List[str]      # files changed by last applied patch (git diff)
    consecutive_test_failures: int # consecutive cargo test failures after a patch
    # Clippy loop safety tracking
    clippy_error_count: Optional[int]        # latest clippy error count
    best_clippy_error_count: Optional[int]   # best (lowest) count this phase
    clippy_has_syntax_error: Optional[bool]  # critical syntax error flag
    # Multi-phase orchestration
    current_phase: str            # "clippy", "refactoring", "performance", "UCIfeatures"
    phases_todo: List[str]        # remaining phases to execute
    phases_completed: List[str]   # completed phases
    phase_iteration: int          # iteration count within current phase
    attempted_warnings: List[str] # warnings already attempted (to avoid retry loops)
    current_warning_signature: Optional[str]  # signature of warning being fixed
    repair_attempts: int          # number of LLM repair attempts for current patch (resets per patch)
