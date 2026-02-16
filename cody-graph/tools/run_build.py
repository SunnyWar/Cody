import subprocess
import textwrap

from state.cody_state import CodyState


def run_build(state: CodyState) -> CodyState:
    print("[cody-graph] run_build: start", flush=True)
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

    result_state = {
        **state,
        "last_output": output,
        "last_command": "cargo_build",
        "status": status,
    }
    if status != "ok":
        print(f"[cody-graph] run_build: error:\n{output}", flush=True)
    print(f"[cody-graph] run_build: end ({status})", flush=True)
    return result_state
