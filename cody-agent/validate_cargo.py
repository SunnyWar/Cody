import subprocess
import sys

def run_command(cmd, desc):
    print(f"[CHECK] {desc}...")
    result = subprocess.run(cmd, shell=True)
    if result.returncode != 0:
        print(f"[FAIL] {desc} failed.")
        return False
    return True

def validate_cargo():
    if not run_command('cargo build', 'cargo build'):
        return False
    if not run_command('cargo test', 'cargo test'):
        return False
    print("[OK] All checks passed.")
    return True

if __name__ == "__main__":
    success = validate_cargo()
    sys.exit(0 if success else 1)
