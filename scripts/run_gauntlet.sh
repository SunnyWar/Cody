#!/usr/bin/env bash
set -e

NEW=$1
OLD=$2
GAMES=$3
OUT=$4

# Simple gauntlet runner using cutechess-cli
cutechess-cli \
  -engine cmd=$NEW name=new \
  -engine cmd=$OLD name=old \
  -each proto=uci tc=40/1 \
  -games $GAMES \
  -rounds $GAMES \
  -pgnout gauntlet.pgn \
  -concurrency 2 \
  -sprt elo0=0 elo1=5 alpha=0.05 beta=0.05 \
  > gauntlet.log

# Convert cutechess output to JSON
python3 <<EOF
import json, sys
log = open("gauntlet.log").read()
result = {"log": log}
open("$OUT", "w").write(json.dumps(result))
EOF
