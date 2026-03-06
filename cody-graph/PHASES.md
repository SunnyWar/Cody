# Multi-Phase Orchestration Guide

The cody-graph system now supports multi-phase orchestration. This allows the system to progress through multiple improvement phases (clippy, refactor, performance, features) automatically.

> **Note:** CLI phase names (e.g., `python main.py refactor`) are aliases for internal phase names used in configuration files (e.g., `"refactoring"` in config.json). See [Running Multi-Phase Orchestration](#running-multi-phase-orchestration) for the complete CLI command reference.

## Current Architecture

### State Management
The `CodyState` now tracks:
- `current_phase` - Which phase is executing
- `phases_todo` - Remaining phases to execute
- `phases_completed` - Phases that have successfully completed
- `phase_iteration` - Iteration counter within current phase

### Graph Flow
```
START → run_clippy → [fix loop] → run_build → run_tests → phase_complete
                                                              ↓
                                              [more phases?] → run_clippy (next phase)
                                                              ↓
                                              [no more] → END
```

## Adding a New Phase

### Step 1: Add Phase to Config
In `cody-agent/config.json`, add a model assignment for your phase:

```json
{
    "models": {
        "clippy": "gpt-5-mini",
        "refactoring": "gpt-5.1",
        "UCIfeatures": "gpt-5.1",
        "performance": "o3",        // Complex optimizations
        "ELOGain": "o3",            // Chess-specific improvements
        "unit_tests_docs": "gpt-5-nano"
    }
}
```

### Step 2: Add System Prompt
In `agents/clippy_agent.py`, update `_get_system_prompt_for_phase()`:

```python
def _get_system_prompt_for_phase(phase: str) -> str:
    phase_prompts = {
        "clippy": "...",
        "refactoring": """
You are Cody's RefactoringAgent...
""",
        "performance": """
You are Cody's PerformanceAgent...
""",
        # Add more phases here
    }
    return phase_prompts.get(phase, phase_prompts["clippy"])
```

### Step 3: Create Phase-Specific Agent (Optional)
For complex phases with different logic, create a new agent file:

```python
# agents/refactoring_agent.py
def refactoring_agent(state: CodyState) -> CodyState:
    # Custom analysis/execution logic for refactoring phase
    ...
```

### Step 4: Wire Into Graph (Optional)
Update `graph/cody_graph.py` to use phase-specific agents:

```python
def after_phase_complete(state: CodyState):
    phase = state["current_phase"]
    if phase == "refactoring":
        return "refactoring_agent"
    elif phase == "performance":
        return "performance_agent"
    # ... etc
    return END
```

## Running Multi-Phase Orchestration

### Run All Configured Phases
```powershell
python .\cody-graph\main.py all
```
This loads phases from `cody-agent/config.json` and executes them in order.

### Run a Single Phase
```powershell
# CLI phase names (aliases for internal phase names)
python .\cody-graph\main.py clippy      # Fix compiler warnings
python .\cody-graph\main.py refactor    # Code quality improvements  
python .\cody-graph\main.py features    # New features/UCI commands
python .\cody-graph\main.py performance # Speed optimization
python .\cody-graph\main.py elogain     # Chess ELO improvements
python .\cody-graph\main.py tests       # Test coverage & docs
```

### Check Phase Progress
Monitor console output for phase transitions:
```
[cody-graph] [DIAG] Phase 'clippy' completed
[cody-graph] [DIAG] Transitioning to phase: refactoring
```

### Resume Interrupted Runs
The orchestrator state is saved to `orchestrator_state.json` after each phase. Future runs can detect and resume from that state.

## Phase Lifecycle

Each phase follows this pattern:
1. **Analysis** - Detect issues/opportunities
2. **Execution** - Apply fixes via LLM-generated diffs
3. **Validation** - Build, test, and verify

```
phase_start → run_clippy → [fix loop until OK] → run_build → run_tests → phase_complete
```

If any step fails:
- Build failure → rollback
- Test failure → rollback
- On rollback → end phase with error

## Phase Examples

### Clippy Phase (Current)
- **Input**: Clippy warnings
- **Process**: Fix one warning at a time
- **Output**: All clippy warnings eliminated

### Refactoring Phase (Planned)
- **Input**: Code quality metrics
- **Process**: Improve structure, readability, maintainability
- **Stop**: No more quality improvements found

### Performance Phase
- **Input**: Benchmark results, profiling data
- **Process**: Optimize hot paths, reduce allocations, improve critical algorithms
- **Model**: o3 (complex optimization reasoning)
- **Stop**: Diminishing returns (<5% improvement)

### ELOGain Phase
- **Input**: Engine playing strength metrics, evaluation scores
- **Process**: Improve chess-specific logic (evaluation, search pruning, move ordering, extensions)
- **Model**: o3 (deep chess knowledge required)
- **Stop**: No more high-impact ELO improvements found
- **Focus Areas**:
  - Evaluation function tuning
  - Search enhancements (LMR, null-move pruning, extensions)
  - Move ordering improvements
  - Endgame knowledge

### UCIfeatures Phase
- **Input**: UCI protocol requirements and missing commands
- **Process**: Implement missing UCI commands or extend existing ones for full tournament-grade UCI support
- **Priority**: Commands most used in tournaments (time management, search options, info output)
- **Stop**: UCI protocol fully supported or requested commands completed

## Logging and Diagnostics

Each phase generates diagnostic logs in `.cody_logs/`:
- `*_clippy_output.txt` - Phase-specific tool output
- `*_llm_response.txt` - LLM reasoning and diffs
- `*_diff_extracted.log` - Applied changes
- `*_build_output.txt` - Build results

## State Persistence

After orchestration completes, the final state is saved to `orchestrator_state.json`:

```json
{
  "current_phase": "clippy",
  "phases_completed": ["clippy"],
  "phases_todo": ["refactoring", "performance"],
  "phase_iteration": 3,
  "status": "ok",
  "last_update": "2026-03-03T12:34:56.789012"
}
```

This allows:
- Tracking which phases have completed
- Resuming from interruption points
- Analyzing orchestration progress

## Troubleshooting

### Phase won't advance
Check `.cody_logs/` for which step failed (clippy, build, or tests).

### LLM generates invalid diffs
Review `.cody_logs/*_llm_response.txt` to verify the system prompt is appropriate for the phase.

### Phase completes but tools still report errors
Check that phase-specific tool validation (e.g., benchmarks for performance phase) is being performed.

## ELO Gain Phase: Chess Engine Improvement Loop

The **ELOGain** phase is a specialized sub-orchestration designed to automatically improve the engine's playing strength. Unlike simpler phases (clippy, refactoring), it implements a tight feedback loop:

```
Candidate Generation → Compilation → Gauntlet Match → Statistical Analysis → Decision (Commit/Revert)
```

Each iteration attempts one improvement. If ELO gain > 0, the change is committed. If ≤ 0, the candidate is reverted and failure analysis is fed back to the LLM for the next iteration.

### Sub-Phases

1. **Candidate Generation** (`elo_gain_candidate_generation()`)
   - LLM proposes a chess-specific improvement (e.g., Null Move Pruning, better evaluation)
   - Output: Unified diff ready to apply

2. **Compilation Validation** (`elo_tools/validate_compilation.py`)
   - Build: `cargo build --release`
   - Perft: `cargo run --release -- perft 5` (verify move generation)
   - Clippy: Non-fatal warning scan

3. **Gauntlet Match** (`elo_tools/gauntlet_runner.py`)
   - Run 50–100 games at fast time control (10s + 0.1s increment)
   - Candidate vs. Stable baseline
   - Output: PGN file with all games

4. **Statistical Analysis** (`elo_tools/analyze_statistics.py`)
   - Parse PGN for match results
   - Calculate ELO difference using Bayesian framework
   - Compute 95% credible interval (error bar)
   - Determine statistical significance

5. **Decision & Commit/Revert** (`elo_tools/commit_or_revert.py`)
   - If ΔElo > 0: Commit with git, update stable baseline
   - If ΔElo ≤ 0: Revert, analyze loss patterns, store for LLM learning

### Integration into Config

In `cody-agent/config.json`, enable the ELO Gain phase:

```json
{
  "phases": ["clippy", "ELOGain"],
  "models": {
    "ELOGain": "o3"  // Use powerful model for chess improvements
  }
}
```

### Configuration

Optional environment variables:
```bash
CODY_ELO_TIME_CONTROL="10+0.1"        # Time control for gauntlet games
CODY_ELO_GAUNTLET_GAMES="50"          # Number of games (default 50)
CODY_ELO_MAX_ITERATIONS="10"          # Max improvement attempts
CODY_ELO_THRESH_ELO="0.0"             # Threshold to commit (default 0.0)
```

### Current Status

- ✅ **Scaffolding Complete**: Main agent (`elo_gain_agent.py`) and placeholder sub-scripts created
- ⏳ **Implementation Phases**:
  - `validate_compilation.py` — Ready for implementation
  - `gauntlet_runner.py` — High priority (cutechess-cli integration)
  - `analyze_statistics.py` — Medium priority (Bayesian ELO calculator)
  - `commit_or_revert.py` — Medium priority (Git + loss analysis)

### How to Run

Once implementations are complete:

```bash
# Run orchestrator with ELO Gain phase enabled
python cody-graph/main.py
```

Monitor progress in console and check `.cody_logs/elo_phase_*.log` for details.

### For Detailed Architecture

See [ELO_GAIN_PHASE.md](ELO_GAIN_PHASE.md) for comprehensive documentation including:
- Detailed workflow diagrams
- Algorithm descriptions
- Implementation roadmap
- Perft validation node counts
- References and tools

## Future Enhancements

- [ ] Per-phase iteration limits
- [ ] Conditional phase branching based on results
- [ ] Phase-specific rollback strategies
- [ ] Parallel phase execution (with conflict detection)
- [ ] Phase result aggregation and reporting
- [ ] ELO Gain phase: LLM feedback loop for failure analysis
- [ ] ELO Gain phase: Adaptive game count (more games if close call)
