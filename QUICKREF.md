# Cody AI Automation - Quick Reference

## File Locations

```
Main entry:      cody-graph/main.py
Graph logic:     cody-graph/graph/cody_graph.py
Agents:          cody-graph/agents/clippy_agent.py
Tools:           cody-graph/tools/*.py
Config:          cody-agent/config.json
State tracking:  orchestrator_state.json (generated)
Diagnostics:     .cody_logs/ (generated)
```

## Quick Commands

### Run the Automated Improvement Agent

```powershell
# From repo root - run all configured phases
python .\cody-graph\main.py all

# Or run a single phase
python .\cody-graph\main.py clippy      # Fix compiler warnings
python .\cody-graph\main.py refactor    # Code quality improvements
python .\cody-graph\main.py features    # New features/UCI commands
python .\cody-graph\main.py performance # Speed optimization
python .\cody-graph\main.py elogain     # Chess ELO improvements
python .\cody-graph\main.py tests       # Test coverage & docs
```

This will:
1. Load configured phases from `cody-agent/config.json`
2. Execute improvements sequentially (or run a single phase)
3. Generate diagnostic logs in `.cody_logs/`
4. Save progress to `orchestrator_state.json`

### Build and Test

```powershell
# Build
cargo build --release

# Run tests
cargo test
cargo test -p bitboard
cargo test -p engine

# Run benchmarks
cargo bench -p engine

# Validate move generation
cargo run --release -p engine -- perft 5

# Play interactively (UCI protocol)
cargo run -p engine
```

### Release Selfplay Gate (ELO Acceptance)

Use this when deciding whether a code change improved playing strength.

```powershell
# 1) Build current candidate
cargo build --release

# 2) Ensure baseline exists in same folder as candidate
# (baseline should come from a previous known-good version)
Copy-Item .\target\release\cody.exe .\target\release\cody-v1.0.exe

# 3) Run reproducible A/B match
Push-Location .\target\release
D:/Cody/.venv/Scripts/python.exe ../../cody-graph/tools/selfplay.py --mode strict
Pop-Location

# If already in repo root (D:\Cody), this is equivalent:
D:/Cody/.venv/Scripts/python.exe .\cody-graph\tools\selfplay.py --mode strict
```

Note: `../../cody-graph/tools/selfplay.py` only works when current directory is `target\release`.

Expected layout in `target/release`:
- Candidate: `cody.exe`
- Baseline: `cody-v*.exe` (highest version is selected)

Acceptance criteria:
- `cargo build --release` passes
- `cargo test -p bitboard` and `cargo test -p engine` pass
- Strict selfplay completes without illegal move/disqualification
- Candidate beats baseline in final score (`W > L`)
- Keep `temp_match.meta.json` and `temp_match.pgn` for reproducibility

### Check Diagnostics

```powershell
# List generated logs
ls .cody_logs/ | tail -10

# View recent LLM response
cat .cody_logs/*_llm_response.txt | tail -100

# View recent clippy output
cat .cody_logs/*_clippy_output.txt | tail -50
```

## Architecture Overview

```
┌─────────────────────────────────────────┐
│  cody-graph (LangGraph Orchestration)   │
├─────────────────────────────────────────┤
│ ✓ run_clippy    (cargo clippy)          │
│ ✓ clippy_agent  (LLM fixes warnings)    │
│ ✓ apply_diff    (patch tool)            │
│ ✓ run_build     (cargo build)           │
│ ✓ run_tests     (cargo test)            │
│ ✓ rollback      (undo on failure)       │
│ ✓ phase_complete (next phase routing)   │
└─────────────────────────────────────────┘
           ↓ (phases ready to implement)
       refactoring, performance, features
```

## Environment Setup

```powershell
# Install dependencies
pip install -U langgraph openai

# Set OpenAI API key
$env:OPENAI_API_KEY = "sk-..."

# Optional: Override repo path
$env:CODY_REPO_PATH = "D:\Cody"
```

## Configuration

Edit `cody-agent/config.json`:

```json
{
  "model": "gpt-5.1",
  "models": {
    "clippy": "gpt-5-mini",
    "refactoring": "gpt-5.1",
    "features": "gpt-5.1",
    "performance": "o3",
    "ELOGain": "o3",
    "unit_tests_docs": "gpt-5-nano"
  },
  "use_local": false
}
```

The system loads all configured models and executes them as phases sequentially.

## Current Workflow

```
START
  ↓
route_phase
  ↓
[Phase == clippy?]
  ├─→ run_clippy (detect pedantic warnings)
  │     ↓
  │   [Warning found?]
  │     ├─→ clippy_agent (LLM proposes fix)
  │     │     ↓
  │     │   apply_diff (patch code)
  │     │     ↓
  │     │   run_clippy (verify fix)
  │     │     ↓
  │     │   [Loop back if more warnings]
  │     │
  │   run_build (compile)
  │     ↓
  │   run_tests (validate)
  │     ↓
  │   phase_complete → next phase or END
  │
  └─→ [Other phases: refactor, performance, etc.]
        ↓
      clippy_agent (LLM improvements, no clippy check)
        ↓
      apply_diff (patch code)
        ↓
      run_build (compile)
        ↓
      run_tests (validate)
        ↓
      phase_complete → next phase or END
```

**Note:** Only the **clippy phase** runs `cargo clippy --W clippy::pedantic`. 
Other phases (refactoring, performance, etc.) skip clippy checks and use the LLM agent directly.

## Phase System (Multi-Phase Ready)

### Available Phases

**CLI Phase Names** (use these in commands):
- `clippy` - Fixes compiler warnings (always runs first)
- `refactor` - Code quality improvements
- `features` - New capabilities / UCI commands
- `performance` - Speed optimizations on critical paths (uses o3)
- `elogain` - Chess-specific improvements for playing strength (uses o3)
- `tests` - Unit test coverage & documentation

**Internal Phase Names** (in config files and code):
- `clippy` → CLI: `clippy`
- `refactoring` → CLI: `refactor`
- `UCIfeatures` → CLI: `features`
- `performance` → CLI: `performance`
- `ELOGain` → CLI: `elogain`
- `unit_tests_docs` → CLI: `tests`
- **unit_tests_docs** - Test coverage and documentation

### Run Specific Phases
```powershell
python .\cody-graph\main.py all          # Run all phases
python .\cody-graph\main.py clippy       # Run only clippy
python .\cody-graph\main.py performance  # Run only performance
python .\cody-graph\main.py ELOGain      # Run only ELOGain
python .\cody-graph\main.py tests        # Run only unit_tests_docs
```

See [cody-graph/PHASES.md](cody-graph/PHASES.md) for phase implementation guide.

## Quality Gates

Every change must pass:
- ✅ clippy (no warnings as errors)
- ✅ `cargo build --release` (compiles)
- ✅ `cargo test` (all tests pass)
- ✅ `cargo run --release -- perft 5` (move gen valid)

## Git Workflow

After successful agent run:
```powershell
# Changes are automatically validated but NOT committed
# Review changes
git diff

# If satisfied, commit manually
git add .
git commit -m "Automated improvement: <phase> phase"
git log --oneline
```

## Diagnostics

The system generates detailed logs for debugging:

**Location:** `.cody_logs/` (with timestamps)

**Key files:**
- `*_clippy_output.txt` - Clippy warnings/errors
- `*_llm_response.txt` - LLM system prompt + context + response
- `*_diff_extracted.log` - Extracted unified diff
- `*_build_output.txt` - Build results
- `*_test_output.txt` - Test execution results
- `*_patch_stderr.log` - Patch tool errors (if any)

**Review recent run:**
```powershell
ls .cody_logs/ | tail -20
cat .cody_logs/*_llm_response.txt
```

See [cody-graph/DIAGNOSTICS.md](cody-graph/DIAGNOSTICS.md) for troubleshooting.

## State Tracking

Current phase and progress saved to `orchestrator_state.json`:

```json
{
  "current_phase": "performance",
  "phases_completed": ["clippy"],
  "phase_pool": {
    "refactoring": 1.0,
    "ELOGain": 5.0,
    "UCIfeatures": 1.0
  },
  "phase_iteration": 5,
  "status": "ok",
  "last_update": "2026-03-03T12:34:56.789012"
}
```

**Note:** Phases are now selected using weighted random selection. `performance` and `ELOGain` have 5x higher weights (5.0) than other phases (1.0), making them more likely to be selected.

## Troubleshooting

| Problem | Solution |
|---------|----------|
| "No API key" | Set `$env:OPENAI_API_KEY = "sk-..."` |
| Patches not applying | Check `.cody_logs/*_diff_extracted.log` for format |
| Build fails repeatedly | Check `.cody_logs/*_build_output.txt` for errors |
| Tests fail after patch | Check `.cody_logs/*_test_output.txt` |
| Wrong model being used | Verify `cody-agent/config.json` has correct model names |

## Architecture Constraints (MUST RESPECT)

- **Fixed-block arena**: Search nodes preallocated, no dynamic allocation in hot path
- **Allocation-free moves**: No Vec/String per-node in search
- **External dependencies**: Allowed only when they are extremely high-performance and used in performance-critical paths
- **Type safety**: Use newtypes (Ply, Depth, Square) not raw integers

See [architecture.md](architecture.md) for details.

## Success Indicators

After `python .\cody-graph\main.py` completes:
- ✅ Repo still builds: `cargo build --release` succeeds
- ✅ All tests pass: `cargo test` passes
- ✅ No clippy warnings: `cargo clippy --` shows none
- ✅ Move generation valid: `cargo run --release -- perft 5` matches expected
- ✅ Diagnostics logged: `.cody_logs/` has timestamped files
- ✅ State saved: `orchestrator_state.json` shows progress

## Resources

- **Chess Programming Wiki**: https://www.chessprogramming.org/
- **UCI Protocol**: https://www.wbec-ridderkerk.nl/html/UCIProtocol.html
- **Stockfish**: https://github.com/official-stockfish/Stockfish
- **Leela Chess Zero**: https://github.com/LeelaChessZero/lc0
- **LangGraph Docs**: https://langchain-ai.github.io/langgraph/

## Next Steps

1. **To improve code quality:** Implement refactoring phase (see [cody-graph/PHASES.md](cody-graph/PHASES.md))
2. **To optimize performance:** Implement performance phase with benchmark tracking
3. **To add UCI protocol support:** Implement UCIfeatures phase for tournament-grade UCI compliance
4. **To debug issues:** Check diagnostics in `cody-graph/DIAGNOSTICS.md`
