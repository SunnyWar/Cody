import subprocess
import textwrap

from state.cody_state import CodyState

def run_clippy(state: CodyState) -> CodyState:
    print("[cody-graph] run_clippy: start", flush=True)
    repo = state["repo_path"]
    try:
        result = subprocess.run(
            ["cargo", "clippy", "--all-targets", "--all-features"],
            cwd=repo,
            capture_output=True,
            text=True,
            check=False,
        )
        output = textwrap.dedent(f"""
        exit_code: {result.returncode}
        stdout:
        {result.stdout}

        stderr:
        {result.stderr}
        """)
        status = "ok" if result.returncode == 0 else "error"
    except Exception as e:
        output = f"Exception while running clippy: {e}"
        status = "error"

    result_state = {
        **state,
        "last_output": output,
        "last_command": "clippy",
        "status": status,
    }
    print(f"[cody-graph] run_clippy: end ({status})", flush=True)
    return result_state
