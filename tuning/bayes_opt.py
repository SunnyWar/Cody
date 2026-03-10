"""
Bayesian optimization step: suggest new parameters based on results.
Usage: python bayes_opt.py <results.json> <params.json>
"""
import sys
import json

if len(sys.argv) != 3:
    print('Usage: python bayes_opt.py <results.json> <params.json>')
    sys.exit(1)

with open(sys.argv[1], 'r', encoding='utf-8') as f:
    results = json.load(f)

# Dummy optimizer: Replace with Optuna/skopt integration
with open(sys.argv[2], 'r', encoding='utf-8') as f:
    params = json.load(f)

# Example: update all params (including arrays)
for k in params:
    if isinstance(params[k], int):
        params[k] += 1  # Dummy increment for demonstration
    elif isinstance(params[k], str) and params[k].startswith('[') and params[k].endswith(']'):
        # Dummy: parse array, increment each element
        arr = [int(x.strip()) for x in params[k][1:-1].split(',')]
        arr = [x + 1 for x in arr]
        params[k] = '[' + ', '.join(str(x) for x in arr) + ']'

with open(sys.argv[2], 'w', encoding='utf-8') as f:
    json.dump(params, f)
print(f'New params written to {sys.argv[2]}')
