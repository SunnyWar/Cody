#!/usr/bin/env bash
set -e

# Parse named arguments
NEW=""
OLD=""
GAMES=""
OUT=""

while [[ $# -gt 0 ]]; do
  case $1 in
    --engine-new)
      NEW="$2"
      shift 2
      ;;
    --engine-old)
      OLD="$2"
      shift 2
      ;;
    --games)
      GAMES="$2"
      shift 2
      ;;
    --output)
      OUT="$2"
      shift 2
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

# Validate required arguments
if [ -z "$NEW" ] || [ -z "$OLD" ] || [ -z "$GAMES" ] || [ -z "$OUT" ]; then
  echo "Usage: $0 --engine-new <path> --engine-old <path> --games <num> --output <path>"
  exit 1
fi

# Simple gauntlet runner using cutechess-cli
cutechess-cli \
  -engine cmd=$NEW name=new \
  -engine cmd=$OLD name=old \
  -each proto=uci tc=40/1 \
  -games $GAMES \
  -rounds $((GAMES / 2)) \
  -pgnout gauntlet.pgn \
  -concurrency 2 \
  -sprt elo0=0 elo1=5 alpha=0.05 beta=0.05 \
  > gauntlet.log

# Parse cutechess output and convert to JSON with SPRT results
python3 <<EOF
import json
import re

log = open("gauntlet.log").read()

# Parse SPRT result from cutechess-cli output
sprt_status = "unknown"
sprt_llr = None
sprt_bounds = None

# Look for SPRT decision lines
# Cutechess-cli outputs lines like:
# "SPRT: LLR=2.95 [0.00,inf), H0=-0.00, H1=5.00"
# "SPRT: Finished: H1 accepted"
# or "SPRT: Finished: H0 accepted"

for line in log.split('\n'):
    if 'SPRT:' in line:
        # Check for H1 accepted first (improvement)
        if 'H1 accepted' in line:
            sprt_status = "accepted"
        # Check for H0 accepted (rejection/no improvement)
        elif 'H0 accepted' in line:
            sprt_status = "rejected"
        
        # Extract LLR value
        llr_match = re.search(r'LLR=([-\d.]+)', line)
        if llr_match:
            sprt_llr = float(llr_match.group(1))

result = {
    "log": log,
    "sprt_status": sprt_status,
    "sprt_llr": sprt_llr
}

with open("$OUT", "w") as f:
    json.dump(result, f, indent=2)
EOF
