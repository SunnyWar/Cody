"""
Run self-play matches to evaluate engine strength.
Usage: python selfplay.py <engine_path> <games> <output.json>
"""
import sys
import subprocess
import json

if len(sys.argv) != 4:
    print('Usage: python selfplay.py <engine_path> <games> <output.json>')
    sys.exit(1)

engine_path = sys.argv[1]
games = int(sys.argv[2])
outfile = sys.argv[3]

# Dummy self-play: Replace with actual harness
results = {'wins': 0, 'losses': 0, 'draws': games}
# Example: subprocess.run([...])

with open(outfile, 'w', encoding='utf-8') as f:
    json.dump(results, f)
print(f'Self-play results written to {outfile}')
