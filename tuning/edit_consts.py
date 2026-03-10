"""
Edit engine_consts.rs with new parameter values.
Usage: python edit_consts.py <params.json>
"""
import sys
import json
import re

CONST_FILE = '../engine/src/engine_consts.rs'

def update_consts(param_dict):
    with open(CONST_FILE, 'r', encoding='utf-8') as f:
        lines = f.readlines()
    for i, line in enumerate(lines):
        for k, v in param_dict.items():
            pat = rf'(pub const {k}: [^=]+ = )[^;]+(;)'  # Only pub consts
            def repl(m):
                # Handle array constants (stored as string in JSON)
                val = v
                if isinstance(val, str) and val.startswith('[') and val.endswith(']'):
                    val = val
                return f'{m.group(1)}{val}{m.group(2)}'
            lines[i] = re.sub(pat, repl, line)
    with open(CONST_FILE, 'w', encoding='utf-8') as f:
        f.writelines(lines)

def extract_consts():
    consts = {}
    with open(CONST_FILE, 'r', encoding='utf-8') as f:
        for line in f:
            m = re.match(r'pub const ([A-Z0-9_]+): [^=]+ = ([^;]+);', line)
            if m:
                name, value = m.group(1), m.group(2).strip()
                # Serialize arrays as strings for JSON
                if value.startswith('[') and value.endswith(']'):
                    consts[name] = value
                else:
                    try:
                        consts[name] = int(value)
                    except ValueError:
                        try:
                            consts[name] = float(value)
                        except ValueError:
                            consts[name] = value
    return consts

if __name__ == '__main__':
    if len(sys.argv) == 2 and sys.argv[1] == '--extract':
        # Extract constants and write to params.json
        consts = extract_consts()
        with open('params.json', 'w', encoding='utf-8') as f:
            json.dump(consts, f, indent=4)
        print('params.json written from engine_consts.rs')
    elif len(sys.argv) == 2:
        with open(sys.argv[1], 'r', encoding='utf-8') as f:
            params = json.load(f)
        update_consts(params)
        print('engine_consts.rs updated.')
    else:
        print('Usage: python edit_consts.py <params.json> | --extract')
        sys.exit(1)
