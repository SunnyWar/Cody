import subprocess
import textwrap

def run_clippy(state: CodyState) -> CodyState:
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

    return {
        **state,
        "last_output": output,
        "status": status,
    }
