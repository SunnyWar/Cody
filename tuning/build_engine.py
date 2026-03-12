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

    # Copy built engine to baseline.exe in tuning directory
    import shutil
    import os
    src = os.path.abspath(os.path.join(WORKSPACE_ROOT, 'target', 'release', 'cody.exe'))
    dst = os.path.abspath(os.path.join(os.path.dirname(__file__), 'baseline.exe'))
    try:
        shutil.copy2(src, dst)
        print(f'Copied {src} to {dst}')
    except Exception as e:
        print(f'Failed to copy {src} to {dst}: {e}')
        sys.exit(1)
