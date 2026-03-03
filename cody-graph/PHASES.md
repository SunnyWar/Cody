# Multi-Phase Orchestration Guide

The cody-graph system now supports multi-phase orchestration. This allows the system to progress through multiple improvement phases (clippy fixes, refactoring, performance, features) automatically.

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
        "refactoring": "gpt-4o",    // NEW PHASE
        "performance": "gpt-5.1",   // NEW PHASE
        "features": "gpt-5.1"       // NEW PHASE
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

### Run with Default Phases
```powershell
python .\cody-graph\main.py
```
This loads phases from `cody-agent/config.json` and executes them in order.

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

### Performance Phase (Planned)
- **Input**: Benchmark results
- **Process**: Optimize hot paths, reduce allocations
- **Stop**: Diminishing returns (<5% improvement)

### Features Phase (Planned)
- **Input**: Feature requirements
- **Process**: Implement up to 3 features
- **Stop**: 3 features completed or no more features

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

## Future Enhancements

- [ ] Per-phase iteration limits
- [ ] Conditional phase branching based on results
- [ ] Phase-specific rollback strategies
- [ ] Parallel phase execution (with conflict detection)
- [ ] Phase result aggregation and reporting
