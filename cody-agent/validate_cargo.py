import subprocess
import sys

def run_command(cmd, desc):
    print(f"[CHECK] {desc}...")
    # Use capture_output=True to keep the logs clean unless there is an error
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    if result.returncode != 0:
        print(f"[FAIL] {desc} failed.")
        print(f"[ERROR LOG]\n{result.stderr}")
        return False
    return True

def validate_cargo(include_clippy=True):
    if not run_command('cargo build', 'cargo build'):
        return False
    if not run_command('cargo test', 'cargo test'):
        return False
    if include_clippy:
        # -D warnings treats all warnings as errors
        if not run_command('cargo clippy -- -D warnings', 'cargo clippy'):
            return False
    print("[OK] All checks passed.")
    return True

if __name__ == "__main__":
    # Allow skipping clippy for pre-checks if desired
    check_clippy = "--no-clippy" not in sys.argv
    success = validate_cargo(include_clippy=check_clippy)
    sys.exit(0 if success else 1)