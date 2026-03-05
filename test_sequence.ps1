#!/usr/bin/env powershell
# Test the engine with the buggy move sequence

"uci" | Out-Host
"position startpos moves d2d3" | Out-Host
"go depth 3" | Out-Host
"position startpos moves d2d3 d7d6" | Out-Host
"go depth 3" | Out-Host
"position startpos moves d2d3 d7d6 c2c3" | Out-Host
"go depth 3" | Out-Host
"quit" | Out-Host
