# ELO Gain Phase: Sub-Orchestration Architecture

## Overview

The **ELO Gain phase** is a sophisticated multi-step loop designed to automatically improve the chess engine's playing strength (ELO rating). Unlike the simpler Clippy phase (single issue → fix → validate), the ELO Gain phase implements a closed-loop system that:

1. **Generates** candidate improvements (via LLM)
2. **Validates** the code compiles and doesn't break move generation
3. **Measures** ELO gain through a gauntlet match
4. **Analyzes** statistical significance
5. **Commits** or **reverts** based on results
6. **Learns** from failures for the next iteration

This guide documents the architecture, expected workflows, and implementation status of each sub-phase.

---

## High-Level Workflow

```
┌─────────────────────────────────────────────────────────────────┐
│ ELO Gain Phase (Main Orchestration Loop)                        │
└─────────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│ 1. Candidate Generation                                         │
│    LLM proposes chess-specific improvement                      │
│    (Null Move Pruning, Better Evaluation, etc.)                │
│    Output: Unified diff file                                    │
└─────────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│ 2. Compilation & Validation                                     │
│    a) Build: cargo build --release                             │
│    b) Perft: cargo run --release -- perft 5                    │
│    c) Clippy: cargo clippy (non-fatal warnings)                │
│    Output: Pass/Fail status                                     │
└─────────────────────────────────────────────────────────────────┘
                           │
                        Pass?
                      /        \
                    No          Yes
                     │            │
                 REVERT      Continue
                     │            │
                     └────────────▼
                                   │
                        ┌─────────────────────────────────────────┐
                        │ 3. The Gauntlet                         │
                        │    Run 50–100 games vs. Stable version  │
                        │    Time control: 10s + 0.1s increment   │
                        │    Output: PGN file + match statistics   │
                        └─────────────────────────────────────────┘
                                   │
                                   ▼
                        ┌─────────────────────────────────────────┐
                        │ 4. Statistical Analysis                 │
                        │    Calculate ELO difference ± error bar  │
                        │    Use Bayesian framework (cutechess or  │
                        │    scipy-based analysis)                │
                        │    Output: ELO gain, significance level  │
                        └─────────────────────────────────────────┘
                                   │
                                   ▼
                        ┌─────────────────────────────────────────┐
                        │ 5. Decision & Commit/Revert             │
                        │    If ΔElo > 0: Commit (update stable)  │
                        │    Else: Revert + analyze failures      │
                        │    Output: Git commit or rollback       │
                        └─────────────────────────────────────────┘
                                   │
                          ┌────────┴──────────┐
                       Commit            Revert
                          │                 │
                          │          Analyze Losses
                          │                 │
                          │        (Feed to LLM for
                          │         next iteration)
                          │                 │
                          └────────┬────────┘
                                   │
                                   ▼
                        ┌─────────────────────────────────────────┐
                        │ Loop: Next Iteration (1–10 attempts)    │
                        │ Or END if max iterations reached        │
                        └─────────────────────────────────────────┘
```

---

## Sub-Phase Details

### 1. Candidate Generation

**File:** `agents/elo_gain_agent.py` → `elo_gain_candidate_generation()`

**Purpose:**
Generate a concrete, testable improvement to the engine. The LLM analyzes:
- Current engine architecture (search, evaluation, move ordering)
- Known weaknesses or missed optimizations
- Recent game analysis (if available)

**Expected Output:**
- Unified diff file ready to apply
- Description of improvement
- Expected impact (e.g., "Null Move Pruning should save ~30% at depth 8+")

**Implementation Status:** ⏳ PLACEHOLDER
- [ ] Integrate with OpenAI/Anthropic LLM
- [ ] Provide engine code context
- [ ] Generate and validate diff format
- [ ] Store candidate in state

---

### 2. Compilation & Validation

**File:** `elo_tools/validate_compilation.py`

**Purpose:**
Ensure the candidate doesn't break the codebase. Three checks:

1. **Build**: `cargo build --release` — Must compile without errors
2. **Perft**: `cargo run --release -- perft 5` — Move generation must be correct
3. **Clippy**: `cargo clippy` — Any warnings are logged (non-fatal)

**Expected Output:**
```json
{
  "build_success": true,
  "perft_depth": 5,
  "perft_pass": true,
  "clippy_warnings": 0
}
```

**Implementation Status:** ⏳ PLACEHOLDER
- [ ] Invoke cargo build with timeout handling
- [ ] Parse perft output; compare against known node counts
- [ ] Validate with release mode (critical for performance)
- [ ] Return structured pass/fail result

**Known Perft Node Counts (for validation):**
```
Position: startpos (rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1)
  Perft 1: 20
  Perft 2: 400
  Perft 3: 5,362
  Perft 4: 71,674
  Perft 5: 809,099
  Perft 6: 9,132,484  (slower, can use 5 for automation)
```

---

### 3. The Gauntlet

**File:** `elo_tools/gauntlet_runner.py`

**Purpose:**
Run a match between the candidate and stable engine binaries. Output a PGN file with all games and basic match stats.

**Configuration:**
- **Game Count:** 50–100 (default 50, fast turnaround)
- **Time Control:** 10 seconds + 0.1s increment (fast autoplay)
- **Alternation:** Candidate plays both white and black
- **Output Format:** Standard PGN with game results

**Expected Output:**
```json
{
  "candidate_wins": 18,
  "stable_wins": 15,
  "draws": 17,
  "candidate_score_percent": 51.5,
  "pgn_file": "/path/to/gauntlet_result.pgn",
  "games_played": 50
}
```

**Implementation Status:** ⏳ PLACEHOLDER (High Priority)
- [ ] Integrate with **cutechess-cli** (preferred) or similar tournament tool
- [ ] Build and locate both binaries (candidate + stable)
- [ ] Configure UCI engine settings
- [ ] Run match with time control + match parameters
- [ ] Parse live/final output for statistics
- [ ] Write PGN to disk
- [ ] Handle timeouts and crashes gracefully

**Tools to Consider:**
- **cutechess-cli**: Industry standard, handles pairing, PGN writing, Bayesian ELO
  - Command: `cutechess-cli -engine name=Cody cmd=./cody ... -variant standard -games 50 -pgnout result.pgn`
- **python-chess**: Pure Python, lighter but slower
- **uci-orchestrator**: Custom UCI mux (could write if needed)

---

### 4. Statistical Analysis

**File:** `elo_tools/analyze_statistics.py`

**Purpose:**
Calculate the ELO difference between engines with confidence intervals. This determines whether the improvement is "real" or just noise.

**Algorithm:**
1. **Scale-Free Win Rate**: Count (wins + 0.5 × draws) / total games
2. **Bayesian ELO Calculation**:
   - Prior: Uninformative beta prior (α=β=1)
   - Likelihood: Binomial (wins / total)
   - Posterior: Beta distribution
   - Point estimate: Mode of posterior
   - Error bar: Credible interval (e.g., 95%)
3. **Result**: ELO difference ± error bar

**Expected Output:**
```json
{
  "elo_difference": 25.3,
  "elo_error_bar": 18.5,
  "candidate_wins": 18,
  "stable_wins": 15,
  "draws": 17,
  "total_games": 50,
  "candidate_score_percent": 51.5,
  "statistically_significant": false
}
```

**Interpretation:**
- **Significant if:** `|elo_diff| > 1.96 × error_bar` (95% confidence)
- **Example:** +25 ± 18 is **not** significant (overlaps 0)
- **Example:** +40 ± 10 **is** significant (well above 0)

**Implementation Status:** ⏳ PLACEHOLDER
- [ ] Parse PGN file for game results and metadata
- [ ] Calculate score percentages correctly
- [ ] Implement Bayesian ELO calculation:
  - Use scipy (preferred) or pure NumPy
  - Beta distribution posterior sampling
  - MCMC for higher accuracy (optional)
- [ ] Validate against cutechess-cli if available
- [ ] Return structured result with confidence intervals

**References:**
- Bayes ELO rating: [KNSB](http://rybka.net/ratings/knsb.html)
- Credible intervals: Likelihood ratio method or MCMC
- Approximation: Simple proportional error (if scipy unavailable)

---

### 5. Decision & Commit/Revert

**File:** `elo_tools/commit_or_revert.py`

**Purpose:**
Final decision logic:
- If **ΔElo > 0** (or > configurable threshold): **Commit** and update baseline
- If **ΔElo ≤ 0**: **Revert** and analyze failure modes for LLM learning

**Commit Workflow:**
1. Stage modified files: `git add bitboard/ engine/`
2. Commit with message: `"ELOGain: [description] (+X.X ELO)"`
3. Tag: `v1.0.1-elo{N}` (or semantic version bump)
4. Update `stable` branch pointer (or merge to main)
5. Update orchestrator state for next iteration

**Revert Workflow:**
1. Clean working directory: `git reset --hard HEAD`
2. Analyze loss games from PGN:
   - Identify positions where candidate lost
   - Extract patterns (openings, pawn structures, tactics)
   - Generate failure analysis text
3. Store analysis in state for next LLM iteration

**Expected Output (Commit):**
```json
{
  "action": "committed",
  "elo_gain": 25.3,
  "message": "Successfully committed: Null Move Pruning (+25.3 ELO)",
  "analysis": null
}
```

**Expected Output (Revert):**
```json
{
  "action": "reverted",
  "elo_gain": -5.2,
  "message": "Candidate did not improve ELO, reverted: Bad Eval Tweak",
  "analysis": "Engine struggled in queen endgames. Eval adjustment penalized passed pawns too heavily..."
}
```

**Implementation Status:** ⏳ PLACEHOLDER
- [ ] Git operations: `add`, `commit`, `tag`, `branch update`
- [ ] Loss analysis: Parse PGN, extract loss games, identify patterns
- [ ] Error handling: Merge conflicts, dirty working directory
- [ ] State persistence: Update `orchestrator_state.json` with new baseline

---

## Integration with Main Orchestrator

The ELO Gain phase is called from the main langgraph orchestration in `cody_graph.py`.

**When triggered:**
1. Phase is selected in `cody-agent/config.json`
2. Main loop routes to `elo_gain_agent()` from `agents/elo_gain_agent.py`
3. Agent manages internal state machine (iterate through 5 sub-phases)
4. After each iteration completes (commit or revert), agent can loop again (up to 10 iterations)

**State Variables (in `CodyState`):**
```python
{
    # ELO Gain phase progress
    "elo_phase_stage": "candidate_generation" | "compilation" | "gauntlet" | "statistical_check" | "decision" | "complete",
    "elo_iterations": int,  # How many candidates tried (0–10)
    "elo_max_iterations": int,  # Cap at 10 by default
    
    # Candidate tracking
    "elo_proposed_candidate": str,  # Description of improvement
    "elo_gauntlet_games": int,  # Number of games to play (default 50)
    
    # Results
    "elo_gauntlet_pgn": str,  # Path to PGN file
    "elo_match_stats": dict,  # Game counts
    "elo_gain_value": float,  # ELO difference (ΔElo)
    "elo_error_bar": float,  # Bayesian error bar
    "elo_phase_outcome": "committed" | "reverted",  # Final decision
    "elo_improvement_committed": float,  # ELO gain of committed version
    "elo_failure_analysis": str,  # Analysis if reverted
}
```

---

## Configuration

In `cody-agent/config.json`:

```json
{
  "models": {
    "ELOGain": "o3"  // Use powerful model for chess improvements
  }
}
```

Optional environment variables:
```bash
# Time control for gauntlet (format: "minutes+increment")
CODY_ELO_TIME_CONTROL="10+0.1"

# Number of games per gauntlet (default 50)
CODY_ELO_GAUNTLET_GAMES="100"

# Max iterations of ELO loop (default 10)
CODY_ELO_MAX_ITERATIONS="10"

# Statistical significance threshold (default 0.0)
CODY_ELO_THRESH_ELO="5.0"
```

---

## Implementation Roadmap

### Phase 1: Scaffold (✅ DONE)
- [x] Create `elo_gain_agent.py` with orchestration loop
- [x] Create placeholder scripts (gauntlet, validator, analyzer, commit_or_revert)
- [x] Define state machine and integration points
- [x] Document architecture

### Phase 2: Compilation Validation (⏳ NEXT)
- [ ] Implement `validate_compilation.py` fully
  - [ ] Run `cargo build --release`
  - [ ] Run `cargo run --release -- perft 5`
  - [ ] Validate perft output against known node counts
  - [ ] Timeout handling and error reporting

### Phase 3: Gauntlet Runner (⏳ HIGH PRIORITY)
- [ ] Implement `gauntlet_runner.py`
  - [ ] Integrate cutechess-cli (if available on host)
  - [ ] Or implement pure Python UCI orchestrator
  - [ ] Handle engine startup, timeouts, crashes
  - [ ] Write PGN output
  - [ ] Parse match statistics

### Phase 4: Statistical Analysis (⏳ MEDIUM PRIORITY)
- [ ] Implement `analyze_statistics.py`
  - [ ] PGN parser for game results
  - [ ] Bayesian ELO calculation
  - [ ] Error bar computation (95% credible interval)
  - [ ] Significance testing

### Phase 5: Commit/Revert + Loss Analysis (⏳ MEDIUM PRIORITY)
- [ ] Implement `commit_or_revert.py`
  - [ ] Git commit workflow
  - [ ] Loss PGN analysis
  - [ ] Failure mode extraction
  - [ ] State updates

### Phase 6: LLM Integration (⏳ LOWER PRIORITY)
- [ ] Feed failure analysis back to LLM for next iteration
- [ ] Context building: "Here's why the candidate failed..."
- [ ] Prompt engineering for better proposals

### Phase 7: Testing & Tuning (⏳ FINAL)
- [ ] Integration tests for full loop
- [ ] Benchmark: Can we improve from ~1000 ELO → 1200 ELO?
- [ ] Optimize gauntlet time (shorter games, fewer iterations)
- [ ] Fine-tune statistical thresholds

---

## Testing & Debugging

### Running Individual Sub-Phases

**Validate compilation:**
```bash
python cody-graph/elo_tools/validate_compilation.py --repo-path D:\Cody --perft-depth 5
```

**Analyze a PGN (once gauntlet produces one):**
```bash
python cody-graph/elo_tools/analyze_statistics.py \
  --pgn results/gauntlet.pgn \
  --candidate-name "Cody Candidate" \
  --stable-name "Cody Stable"
```

**Test commit/revert logic:**
```bash
python cody-graph/elo_tools/commit_or_revert.py \
  --repo-path D:\Cody \
  --elo-gain 15.5 \
  --candidate-description "Null Move Pruning" \
  --threshold 0.0
```

### Running Full ELO Phase

```bash
python cody-graph/main.py
```
(Once `cody-agent/config.json` includes `"ELOGain"` in `phases`.)

---

## Diagnostics & Logs

All phase activities are logged to `.cody_logs/` directory:
- `elo_phase_{iteration}.log` — Full transcript per iteration
- `gauntlet_results_{iteration}.pgn` — Game records
- `analysis_{iteration}.json` — Statistical results
- `decision_{iteration}.json` — Commit/revert decision

---

## Known Limitations & Future Improvements

1. **Stable Baseline**: Currently assumes a prebuilt stable binary. Future: Auto-build from tag/branch.
2. **Time Control**: Fixed at 10s + 0.1s. Future: Configurable via env vars or config.
3. **Game Count**: Only 50 by default (fast but noisy). Future: Adaptive (more games if close call).
4. **Loss Analysis**: Placeholder only. Future: Deep pattern extraction + clustering.
5. **LLM Feedback Loop**: Not yet connected. Future: Feed analysis directly to proposal LLM.

---

## References

- **Bayesian ELO**: Rémi Coulom's KNSB rating system
- **cutechess-cli**: Lucas Chess orchestrator CLI
- **scipy.stats**: Binomial, Beta, Credible intervals
- **Chess move generation**: Perft (Performance Test for move generation)

---

## Questions or Issues?

- Check the main `cody-graph/PHASES.md` for general orchestrator concepts
- See `architecture.md` for engine design and fixed-block allocator details
- Check `orchestrator_state.json` for current state after each phase run
