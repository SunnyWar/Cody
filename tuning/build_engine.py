"""
Build the engine with current constants.
Usage: python build_engine.py
"""
import subprocess
import sys

WORKSPACE_ROOT = '..'

if __name__ == '__main__':
    result = subprocess.run(['cargo', 'build', '--release'], cwd=WORKSPACE_ROOT)
    if result.returncode != 0:
        print('Build failed.')
        sys.exit(1)
    print('Engine built successfully.')
