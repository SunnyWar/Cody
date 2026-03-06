# Build Failure Recovery and Retry Logic Implementation

## Overview

The cody-graph orchestration system now implements a robust **build failure recovery** system that prevents the orchestration from exiting with broken source files. When a patch causes a build failure, the system:

1. **Attempts LLM repair** (up to 2 attempts via `build_repair_attempt`)
2. **Tracks failed warnings** to avoid re-attempting them
3. **Rolls back on exhausted repairs** and continues with the next warning
4. **Detects already-applied patches** to handle edge cases
5. **Provides reusable retry logic** for all phases (not just clippy)

## Key Changes

### 1. New `RetryManager` Class (`tools/retry_manager.py`)

A reusable, phase-agnostic tool for managing retry attempts and failure tracking.

**Capabilities:**
- Track failed warnings per phase (stored in `<phase>_failed_warnings.json` in `.cody_logs/`)
- Determine if repair should be attempted (max 2 attempts per patch)
- Increment/reset repair attempt counters
- Mark warnings as permanently failed for a phase
- Log detailed repair failure information

**Usage Pattern:**
```python
from tools.retry_manager import create_retry_manager

retry_mgr = create_retry_manager(state)
if retry_mgr.should_attempt_repair(state):
    # Attempt repair
    state = retry_mgr.increment_repair_attempts(state)
else:
    # Mark warning as failed and move on
    retry_mgr.mark_warning_failed(
        phase=state["current_phase"],
        warning_signature=state["current_warning_signature"],
        reason="build error after repair attempt"
    )
```

Benefits:
- **Reusable**: Same logic applies to all phases (clippy, refactoring, performance, etc.)
- **Persistent**: Tracks failed warnings per phase across sessions
- **Safe**: Limits repair attempts to prevent infinite loops
- **Observable**: Logs failure reasons for debugging

### 2. Enhanced `run_build` Tool (`tools/run_build.py`)

Updated to detect when repair attempts are needed:

```
run_build execution flow:
  ├─ run cargo build
  ├─ count build errors
  └─ if build failed after a patch:
      ├─ check: should_attempt_repair()?
      ├─ YES → mark state with "build_failed_needs_repair" flag
      └─ NO  → will be routed to rollback
```

Key additions:
- Imports `RetryManager` to check repair feasibility
- Marks build failures for routing to repair attempt vs rollback

### 3. New Graph Node: `build_repair_attempt`

A new node in the orchestration graph that:

1. **Increments repair attempt counter**
2. **Calls `clippy_agent`** with build error context
3. **Routes to `apply_diff`** to apply the repair patch

```
build_repair_attempt node:
  ├─ Increment repair_attempts counter
  ├─ Call clippy_agent with BUILD_REPAIR_MODE context
  └─ clippy_agent generates repair diff
      → apply_diff applies the fix
      → run_build validates it compiles
```

### 4. Updated Router: `after_build` Conditional

The `after_build` router now has three paths:

```python
if status == "ok":
    return "run_tests"  # Build succeeded
elif should_attempt_repair(state):
    return "build_repair_attempt"  # Try LLM repair
else:
    return "rollback_changes"  # Repair exhausted, rollback
```

This prevents immediate rollback of a broken patch—the LLM gets a chance to fix it first.

### 5. Enhanced `after_rollback` Router

When a repair attempt fails and triggers rollback:

```python
if last_command == "cargo_build" and last_diff:
    # Mark the warning as permanently failed
    retry_mgr.mark_warning_failed(
        phase=state["current_phase"],
        warning_signature=state["current_warning_signature"],
        reason=f"Build failed after {repair_attempts} repair attempt(s)"
    )
    # Add signature to attempted_warnings
    # Return to: run_clippy (to try next warning)
else if last_command == "apply_diff":
    # Malformed patch or policy violation
    # Return to: run_clippy (to try next warning)
else:
    # Critical failure during validation
    # Return to: END
```

This ensures that when a patch can't be repaired, we skip it and try the next warning instead of stopping the entire phase.

### 6. Updated State (`state/cody_state.py`)

Added tracking fields:

```python
class CodyState(TypedDict):
    ...
    repair_attempts: int  # Counter for LLM repair attempts on current patch
                          # Resets to 0 when a new patch is applied
                          # Incremented by build_repair_attempt node
```

### 7. Enhanced `clippy_agent` Context

The agent now detects repair mode and provides appropriate context:

```python
is_build_repair = (
    state.get("last_command") == "cargo_build" and 
    int(state.get("repair_attempts", 0) or 0) > 0
)

if is_build_repair:
    context_parts.append(
        f"BUILD REPAIR MODE (Attempt #{repair_attempt_num}):\n"
        f"The previous code change caused the build to fail. "
        f"Generate a minimal fix that resolves the build error."
    )
```

### 8. Updated `apply_diff` Tool

Now resets `repair_attempts` to 0 after successfully applying a new patch:

```python
if patch_applied_successfully:
    return {
        **state,
        "last_diff": diff_content,
        "repair_attempts": 0,  # Fresh patch → fresh repair allocation
        ...
    }
```

**Additional Robustness Improvements:**

- **Already-Applied Detection**: When a patch fails to apply, the system now checks if it's already been applied by:
  1. Attempting reverse git apply (confirms patch is present)
  2. Checking file content for all added lines
  
- **Edge Case Handling**: If a patch is detected as already applied, it treats it as success rather than failure, avoiding spurious rollbacks.

- **Graceful git apply Strategies**: Multiple application strategies with fallback:
  1. Standard `git apply --whitespace=nowarn`
  2. Zero-context hunks: `git apply --unidiff-zero`
  3. Recount mode: `git apply --unidiff-zero --recount`

### 9. Updated `phase_complete` Transition

When moving to the next phase, repair attempts are reset:

```python
if todo:
    return {
        **state,
        "current_phase": next_phase,
        "repair_attempts": 0,  # Reset for new phase
        "attempted_warnings": [],  # Reset for new phase
        ...
    }
```

## Orchestration Flow Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│ Clippy Agent generates patch for warning                         │
└──────────────────────┬──────────────────────────────────────────┘
                       │
                       ▼
           ┌───────────────────────┐
           │   apply_diff          │
           │ resets repair_attempts│
           │   to 0 on success     │
           └───────────────────────┘
                       │
                       ▼
           ┌───────────────────────┐
           │   run_build           │
           │ (validates patch)     │
           └───────────────────────┘
                       │
           ┌───────────┴────────────┐
           │                        │
        SUCCESS                  FAILURE
           │                        │
           ▼                        ▼
      run_tests         ┌──────────────────────┐
                        │ Check: repair_attempts│
                        │ < MAX (currently 2) ?│
                        └──────────┬───────────┘
                                   │
                        ┌──────────┴─────────┐
                        │                    │
                       YES                   NO
                        │                    │
                        ▼                    ▼
          build_repair_attempt         rollback_changes
          │                            │
          ├─ Increment repair_          ├─ Mark warning failed
          │  attempts counter         │  │
          ├─ Call clippy_agent       │  ├─ Add to attempted_
          │  with BUILD_REPAIR_MODE  │  │  warnings
          │                          │  │
          └─ Apply repair patch ─────┘  └─── run_clippy
                                             (try next warning)
```

## Workflow Example: Clippy Phase

```
python .\cody-graph\main.py clippy

BEFORE (Broken):
  [1] clippy finds issue → LLM generates fix → patch applied
  [2] patch breaks build → system exits with broken code ✗

AFTER (Fixed):
  [1] clippy finds issue → LLM generates fix → patch applied
  [2] patch breaks build → run_build fails
  [3] router detects: repair_attempts (0) < MAX (2) → build_repair_attempt
  [4] build_repair_attempt increments counter → calls clippy_agent
  [5] clippy_agent receives BUILD_REPAIR_MODE context → LLM generates fix
  [6] fix applied → run_build validates
      [A] Success → continue to run_tests ✓
      [B] Fails again → repair_attempts (1) < MAX (2) → retry repair_attempt
  [7] Second repair fails → repair_attempts (2) >= MAX (2)
      → rollback_changes → mark warning failed → run_clippy (next warning)
      → all prior warnings remain fixed ✓
```

## Benefits

1. **No Broken Source Files**: Patches that break the build are either fixed or rolled back
2. **Autonomous Recovery**: LLM helps repair its own failures up to MAX_REPAIR_ATTEMPTS
3. **Progressive Improvement**: Failed warnings are skipped; other warnings continue being fixed
4. **Reusable Pattern**: Same RetryManager works for all phases
5. **Trackable**: Failed warnings are logged and persisted per phase
6. **Safe Limits**: MAX_REPAIR_ATTEMPTS (2) prevents infinite loops
7. **Observable**: Detailed routing decisions in logs

## Configuration

Adjust repair attempt limit in [tools/retry_manager.py](tools/retry_manager.py):

```python
class RetryManager:
    MAX_REPAIR_ATTEMPTS = 2  # Change this value
```

## Testing

To verify the system works:

```bash
# Test that graph loads without errors
cd d:\Cody\cody-graph
python -c "from graph.cody_graph import app; print('✓ Graph loads')"

# Run a single phase
python .\cody-graph\main.py clippy

# Monitor .cody_logs/ for:
# - <timestamp>_repair_failure.log   (when repairs fail)
# - clippy_failed_warnings.json      (persistent failure tracking)
# - <timestamp>_build_output.txt     (build validation logs)
```

## Edge Cases Handled

1. **Patch syntax error**: Caught by `apply_diff` validation → next warning
2. **Build error after first repair**: Increments counter → retry
3. **Build error after max repairs**: Marked failed → next warning
4. **Test failure after repair**: Same flow as original patch → runs test repair
5. **Rollback during repair**: Marks warning failed → continues
6. **Phase transition**: Resets repair attempts and attempted_warnings
7. **Already-applied patch**: Detected via reverse-apply or content check → treated as success
8. **Corrupted code state**: Detected via content verification → skips broken warning
9. **Stale clippy output**: System handles patches that target code already partially fixed
10. **Failed rollback**: Attempts continue with next warning despite rollback errors

## Implementation Notes

- **Stateless Retries**: Each build_repair_attempt call is independent
- **Deterministic**: Fixed MAX_REPAIR_ATTEMPTS prevents non-deterministic loops
- **Transparent Logging**: All routing decisions logged to stdout with [DIAG] prefix
- **Backward Compatible**: Doesn't break existing phase agents or tools

## Initial Failure Analysis and Resolution

### What Happened

On first run of `python .\cody-graph\main.py clippy`:

1. **Issue**: Code had a conflict:
   - Line 358: `let mut move_index = 0;` (declared but shadowed)
   - Line 359: `for (move_index, m) in moves_vec.iter().cloned().enumerate() {` (uses enumerate)
   - Line 436: `move_index += 1;` (tries to mutate immutable loop variable)

2. **LLM Fix**: Generated a patch to change line 359 from `for m in ...` to `for (move_index, m) in ...enumerate()` 

3. **Problem**: The code was **already in that state** (partially fixed), so the patch couldn't apply
   - Context mismatch: patch expected `for m in moves_vec...`, found `for (move_index, m) in ...enumerate()`

4. **Result**: Patch apply failed → system tried to rollback → exit

### Root Causes Addressed

1. **Code Corruption**: The mixed state (incomplete refactoring) caused patch application to fail
   - **Fix**: Manually corrected the code by removing the unused declaration and increment
   - **Prevention**: Added content-based patch detection to handle already-applied patches gracefully

2. **Patch Already Applied**: The recovery system didn't handle this edge case
   - **Fix**: Added `_is_patch_already_applied()` to detect when changes are already in place
   - Added reverse-apply check in `_run_patch_with_strategies()` 
   - These detect when a patch is already applied and treat it as success

3. **No Fallback for Detection Failure**: If patch tool fails, system would exit
   - **Fix**: Added Python-based content verification as fallback
   - Even if git apply fails, system can detect and handle already-applied patches

### Improvements Made

**In [tools/apply_diff.py](tools/apply_diff.py):**
- Added `_is_patch_already_applied()` function with two strategies:
  1. Reverse-apply check via git
  2. Content-based verification (scan for added lines)
  
- Modified `_run_patch_with_strategies()` to detect already-applied patches
  
- Enhanced failure handling in main apply_diff function:
  - Check if patch is already applied before marking as failure
  - Treat already-applied patches as success
  - Continue to next warning instead of exiting

### How It Prevents Future Issues

```
If patch fails to apply with "patch does not apply":
  ├─ Check: Can reverse-apply succeed? 
  │  └─ YES → Patch already applied, treat as success ✓
  └─ NO → Check: Are all added lines in file?
     ├─ YES → Content-based detection confirms applied, treat as success ✓  
     └─ NO → Patch legitimately failed, skip warning and try next ✗
```

This prevents:
- **False failures**: Patches that are already applied won't crash the system
- **Broken source**: Even if patches fail, next warnings are attempted  
- **Silent successes**: Already-applied patches are logged and tracked
- **Stuck loops**: Attempted warnings are tracked to avoid rehashing

### Code Fixes Applied

In [engine/src/search/core.rs](engine/src/search/core.rs):
- **Removed** line 358: `let mut move_index = 0;` (unused, shadowed by loop variable)
- **Removed** line 436: `move_index += 1;` (enumerate provides immutable index, increment not needed)

This fixed the compilation errors and allows the system to move on to the next warning (too_many_arguments).
