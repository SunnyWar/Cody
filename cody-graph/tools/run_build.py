import subprocess
import textwrap

from state.cody_state import CodyState


def run_build(state: CodyState) -> CodyState:
    repo = state["repo_path"]
    try:
        result = subprocess.run(
            ["cargo", "build"],
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
        output = f"Exception while running cargo build: {e}"
        status = "error"

    return {
        **state,
        "last_output": output,
        "last_command": "cargo_build",
        "status": status,
    }
