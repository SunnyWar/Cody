# ELO Gain Phase Implementation - Summary & Quick Start

## What Was Created

A complete placeholder architecture for the **ELO Gain phase sub-orchestration loop**. This enables Cody to automatically improve its chess engine strength through a structured feedback loop.

### New Files & Directories

#### Core Agent
- **[agents/elo_gain_agent.py](agents/elo_gain_agent.py)** (~270 lines)
  - Main orchestration agent with internal state machine
  - Routes through 5 sub-phases: Candidate → Compilation → Gauntlet → Analysis → Decision
  - Manages iteration loop (up to 10 attempts per session)

#### ELO Tools Sub-Scripts
- **[elo_tools/](elo_tools/)** — Package directory
  - **[gauntlet_runner.py](elo_tools/gauntlet_runner.py)** — Match orchestration (cutechess-cli integration placeholder)
  - **[validate_compilation.py](elo_tools/validate_compilation.py)** — Build + perft validation
  - **[analyze_statistics.py](elo_tools/analyze_statistics.py)** — Bayesian ELO calculation with error bars
  - **[commit_or_revert.py](elo_tools/commit_or_revert.py)** — Git operations + loss analysis

#### Documentation
- **[ELO_GAIN_PHASE.md](ELO_GAIN_PHASE.md)** (~400 lines) — Comprehensive architecture guide
- **[PHASES.md](PHASES.md)** (updated) — Integration docs + quick reference

#### Graph Integration
- **[graph/cody_graph.py](graph/cody_graph.py)** (updated) — Added routing for ELO Gain phase

---

## Quick Start: Testing the Scaffolding

### 1. Verify Structure
```bash
# Check that new files exist
ls -R cody-graph/elo_tools/
ls cody-graph/agents/elo_gain_agent.py
ls cody-graph/ELO_GAIN_PHASE.md
```

### 2. Enable ELO Gain Phase
Update `cody-agent/config.json`:
```json
{
  "phases": ["clippy", "ELOGain"],
  "models": {
    "clippy": "gpt-5-mini",
    "ELOGain": "o3"
  }
}
```

### 3. Run Orchestrator (Will Use Placeholders)
```bash
python cody-graph/main.py
```

You'll see the orchestrator cycle through clippy (produces real fixes) then enter ELO Gain phase (returns placeholder results since scripts aren't fully implemented yet).

### 4. Check State Files
After running, review:
- Console output shows phase transitions
- `orchestrator_state.json` tracks progress
- `.cody_logs/elo_phase_*.log` contains phase activity

---

## Implementation Roadmap

The following scripts are ready for implementation in priority order:

### Phase 1: Compilation Validator ⏳ NEXT
**File:** `elo_tools/validate_compilation.py`

**What's needed:**
```python
# TODO items in validate_compilation.py:
# 1. Run cargo build --release with timeout
# 2. Run cargo run --release -- perft 5
# 3. Validate perft output against known node counts:
#    - Startpos perft 5: 809,099 nodes
# 4. Return structured pass/fail
```

**Why first:** Quick validation gates all other phases; catch compilation issues early.

### Phase 2: Gauntlet Runner 🔴 HIGH PRIORITY  
**File:** `elo_tools/gauntlet_runner.py`

**What's needed:**
```python
# TODO items in gauntlet_runner.py:
# 1. Check for cutechess-cli (or implement UCI orchestrator)
# 2. Build both candidate + stable binaries
# 3. Configure match: 50 games, 10s + 0.1s increment, alternating colors
# 4. Parse PGN output for: wins, losses, draws, score %
# 5. Return statistics dict
```

**Why high priority:** Everything downstream depends on gauntlet results. This is the "measurement" of engine strength.

### Phase 3: Statistical Analyzer 🟡 MEDIUM
**File:** `elo_tools/analyze_statistics.py`

**What's needed:**
```python
# TODO items in analyze_statistics.py:
# 1. Parse PGN file for game results
# 2. Calculate score ratio: (wins + 0.5*draws) / games
# 3. Bayesian ELO difference:
#    - Option A: Use cutechess-cli if available
#    - Option B: scipy Beta distribution for posterior
# 4. Compute 95% credible interval (error bar)
# 5. Determine significance: |elo_diff| > 1.96 * error_bar
```

**Why medium:** Needed for decision logic, but can use placeholders first to test flow.

### Phase 4: Commit/Revert Handler 🟡 MEDIUM
**File:** `elo_tools/commit_or_revert.py`

**What's needed:**
```python
# TODO items in commit_or_revert.py:
# 1. If ELO > 0:
#    a. git add bitboard/ engine/
#    b. git commit -m "ELOGain: [...] (+X.X ELO)"
#    c. git tag -a v1.0.X-eloN
# 2. If ELO <= 0:
#    a. git reset --hard HEAD
#    b. Parse loss PGNs for patterns
#    c. Generate failure summary for LLM
```

**Why lower priority:** Can function with basic git operations first; loss analysis is nice-to-have for next iteration.

---

## Architecture Overview

### 5-Phase Loop (Flow Chart)

The ELO Gain phase loops until **N=5 successful improvements achieved** or **50 max iterations reached**:

```
┌──────────────────────────────────────────┐
│ Candidate Generation                     │
│ LLM proposes chess improvement           │
│ Output: Unified diff                     │
└──────────────────────────────────────────┘
                    ↓
┌──────────────────────────────────────────┐
│ Compilation Validation                   │
│ cargo build --release && perft 5         │
├──────────────────────────────────────────┤
│ ✅ PASS → Continue                       │
│ ❌ FAIL → REVERT                         │
└──────────────────────────────────────────┘
                    ↓
┌──────────────────────────────────────────┐
│ The Gauntlet (50 games)                  │
│ Candidate vs. Stable at 10s + 0.1s       │
│ Output: PGN + match statistics           │
└──────────────────────────────────────────┘
                    ↓
┌──────────────────────────────────────────┐
│ Statistical Analysis                     │
│ Bayesian ELO: ΔElo ± error_bar          │
│ Determine significance                   │
└──────────────────────────────────────────┘
                    ↓
┌──────────────────────────────────────────┐
│ Decision: Commit or Revert               │
│ If ΔElo > 0: git commit + tag (SUCCESS++)│
│ If ΔElo ≤ 0: git reset + analyze losses │
└──────────────────────────────────────────┘
                    ↓
   ⏁ Continue until:
     • 5 successes achieved (PRIMARY)
     • 50 iterations reached (FAILSAFE)
```

### State Machine (Internal to elo_gain_agent)

```python
stage = state.get("elo_phase_stage", "candidate_generation")

if stage == "candidate_generation":
    return elo_gain_candidate_generation(state)
elif stage == "compilation":
    return elo_gain_compilation_check(state)
elif stage == "gauntlet":
    return elo_gain_gauntlet_match(state)
elif stage == "statistical_check":
    return elo_gain_statistical_check(state)
elif stage == "decision":
    elo_gain_decision(state)
    # If committed: state["elo_successful_commits"] incremented
    # If reverted: state unchanged
    return state
elif stage == "complete":
    state["elo_iterations"] += 1
    # Check if target reached (default N=5 successes)
    if state.get("elo_successful_commits", 0) >= state.get("elo_target_successes", 5):
        return state  # Exit: target achieved
    # Check if max iterations reached (default 50)
    if state["elo_iterations"] >= state.get("elo_max_iterations", 50):
        return state  # Exit: max attempts reached
    # Otherwise: loop back to next iteration
    state["elo_phase_stage"] = "candidate_generation"
    return elo_gain_agent(state)  # Recursive: next iteration
```

---

## Configuration & Environment Variables

### Enable Phase (cody-agent/config.json)
```json
{
  "phases": ["clippy", "ELOGain"],
  "models": {
    "ELOGain": "o3"  // Use powerful model for chess
  }
}
```

### Optional Env Vars
```bash
# Windows PowerShell
$env:CODY_ELO_TIME_CONTROL = "10+0.1"           # Time control
$env:CODY_ELO_GAUNTLET_GAMES = "50"             # Games per match
$env:CODY_ELO_TARGET_SUCCESSES = "5"            # Target improvements (default 5)
$env:CODY_ELO_MAX_ITERATIONS = "50"             # Max attempts (default 50)
$env:CODY_ELO_THRESH_ELO = "0.0"                # Commitment threshold

# Linux/macOS
export CODY_ELO_TIME_CONTROL="10+0.1"
export CODY_ELO_TARGET_SUCCESSES="5"
# ... etc
```

**Target Behavior**: The ELO Gain phase will loop until **5 successful improvements are committed** or **50 iterations are attempted**, whichever comes first.

---

## Testing Individual Components

### Test Compilation Validator (Once Implemented)
```bash
python cody-graph/elo_tools/validate_compilation.py \
  --repo-path D:\Cody \
  --perft-depth 5
```

### Test Gauntlet Runner (Once Implemented)
```bash
python cody-graph/elo_tools/gauntlet_runner.py \
  --candidate ./target/release/cody.exe \
  --stable ./stable/cody.exe \
  --games 50 \
  --time-control "10+0.1" \
  --output results.pgn
```

### Test Statistical Analyzer (Once Implemented)
```bash
python cody-graph/elo_tools/analyze_statistics.py \
  --pgn results.pgn \
  --candidate-name "Cody Candidate" \
  --stable-name "Cody Stable"
```

---

## Expected Improvements & ELO Gains

Once fully implemented, typical ELO gains per improvement:

| Improvement Type | Typical Gain | Examples |
|---|---|---|
| Move Ordering | +10 to +50 ELO | Killer moves, history heuristic |
| Pruning | +15 to +80 ELO | Null move, futility, SEE |
| Evaluation Tuning | +5 to +30 ELO | PST adjustments, piece weights |
| Search Extensions | +5 to +25 ELO | Check extension, passed pawn |
| Time Management | +5 to +20 ELO | Better allocation, search stability |

Target: Improve from ~1000 ELO baseline toward 1200+ ELO through iterative improvements.

---

## Files Modified

- `cody-graph/graph/cody_graph.py` — Added routing for ELO Gain phase
- `cody-graph/PHASES.md` — Added ELO Gain phase documentation

---

## Next Steps

1. **Immediate:** Run `python cody-graph/main.py` with ELOGain enabled (will show placeholders working)
2. **This iteration:** Implement `validate_compilation.py` (quickest win)
3. **Next session:** Implement `gauntlet_runner.py` (critical path)
4. **Future:** Statistical analyzer, commit logic, loss analysis
5. **Polish:** LLM feedback loop, convergence testing, reliability improvements

---

## Documentation Files

- **ELO_GAIN_PHASE.md** — 400-line comprehensive guide with examples
- **PHASES.md** — Quick reference + integration guide
- **This file** — Quick start + roadmap

---

## Questions?

See [ELO_GAIN_PHASE.md](ELO_GAIN_PHASE.md) for:
- Detailed algorithm descriptions
- Perft node count reference
- Tool recommendations (cutechess-cli, scipy)
- Known limitations and future improvements
