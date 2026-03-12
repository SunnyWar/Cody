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
        content = f.read()
    # List of constants that must be integers
    int_consts = [
        "MATERIAL_PAWN", "MATERIAL_KNIGHT", "MATERIAL_BISHOP", "MATERIAL_ROOK", "MATERIAL_QUEEN",
        "MATERIAL_KING", "BISHOP_PAIR_BONUS", "DOUBLED_PAWN_PENALTY", "ISOLATED_PAWN_PENALTY",
        "MOBILITY_WEIGHT", "EXPOSED_KING_PENALTY", "OPEN_FILE_NEAR_KING", "KING_LACKING_ESCAPE_SQUARES",
        "ROOK_ON_OPEN_FILE_BONUS", "ROOK_ON_SEMIOPEN_FILE_BONUS"
    ]
    array_consts = ["PASSED_PAWN_BONUS_BY_ADVANCE", "PAWN_NEAR_PROMOTION"]
    for k, v in param_dict.items():
        pat = rf'(pub const {k}: [^=]+ = )[^;]+(;)'  # Only pub consts
        def repl(m):
            val = v
            if k in int_consts:
                try:
                    val = int(round(float(val)))
                except Exception:
                    pass
            elif k in array_consts:
                # Ensure array values are ints
                if isinstance(val, list):
                    val = '[' + ', '.join(str(int(round(float(x)))) for x in val) + ']'
                elif isinstance(val, str) and val.startswith('[') and val.endswith(']'):
                    try:
                        arr = json.loads(val)
                        val = '[' + ', '.join(str(int(round(float(x)))) for x in arr) + ']'
                    except Exception:
                        pass
            return f'{m.group(1)}{val}{m.group(2)}'
        content = re.sub(pat, repl, content)
    with open(CONST_FILE, 'w', encoding='utf-8') as f:
        f.write(content)

def extract_consts():
    consts = {}
    with open(CONST_FILE, 'r', encoding='utf-8') as f:
        for line in f:
            m = re.match(r'pub const ([A-Z0-9_]+): [^=]+ = ([^;]+);', line)
            if m:
                name, value = m.group(1), m.group(2).strip()
                # Parse arrays as native JSON arrays
                if value.startswith('[') and value.endswith(']'):
                    try:
                        arr = json.loads(value.replace(' ', '').replace('][', '],['))
                        consts[name] = arr
                    except Exception:
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
