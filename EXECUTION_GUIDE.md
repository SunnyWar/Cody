# Orchestration Execution Guide

## Pre-Run Verification

Before running the orchestration, verify basic conditions:

```powershell
# 1. Engine builds cleanly
cd D:\Cody
cargo build --release
# Should complete with "Finished release"

# 2. Sanity check binary exists
ls target/release/cody.exe
# Should show: cody.exe with recent timestamp

# 3. cutechess-cli available
cutechess-cli --version
# Should show version 2.x or higher
```

## Running the Orchestration

```powershell
cd D:\Cody
python .\cody-graph\main.py elogain
```

**Expected duration**: 5-20 minutes
- Sanity check self-play: ~3 minutes (10 games)
- Candidate generation: ~1 minute (LLM call)
- Compilation: ~2-3 minutes (cargo build + test)
- Validation: ~1-2 minutes (cargo test)
- Decision/Commit: ~30 seconds

## Key Output Indicators

### ✅ Success Indicators

Look for these in terminal output:

```
[cody-graph] [ELO Gain] [0/6] Sanity check phase
[cutechess] Playing 10 games (10s+0.1s time control)...
[cody-graph] [ELO Gain] [OK] Sanity check completed: warnings found
[cody-graph] [ELO Gain] [OK] Extracted worst_fail.pgn with 3 problematic games

[cody-graph] [ELO Gain] [1/6] Candidate generation phase
[candidate_generator] Analyzed 3 failing games
[candidate_generator] Inferred bug pattern: illegal_move_generation
[cody-graph] [ELO Gain] Proposed TEST: Position-Specific Illegal Move Test

[cody-graph] [ELO Gain] [2/6] Compilation & Validation phase
[cody-graph] [ELO Gain] [OK] Test 'test_illegal_move_issue_reproduction' added to bitboard/src/lib.rs
[cody-graph] [ELO Gain] [OK] Test code compiles successfully

[cody-graph] [ELO Gain] [3/6] Running Validation
[cody-graph] [ELO Gain] [OK] Unit test PASSED - issue successfully reproduced!

[cody-graph] [ELO Gain] [5/6] Decision phase
[cody-graph] [ELO Gain] [OK] Unit test successfully reproduces issue — COMMITTING
[commit_util] Add regression test for illegal move generation
[cody-graph] [ELO Gain] Version: 0.1.2 → 0.1.3
[cody-graph] [ELO Gain] Copied to C:\chess\Engines\Cody-0.1.3.exe
```

### ⚠️ Warning Indicators

These are NOT failures, just information:

```
[candidate_generator] LLM parsing: fell back to placeholder
  → LLM returned unparseable JSON, using generic test instead
  → Test still works, just not position-specific

[cody-graph] [ELO Gain] worst_fail.pgn not found
[candidate_generator] Analyzed 0 failing games
  → Parsed worst_fail.pgn but it was empty
  → Will use placeholder test template
  → Still generates working test

[cody-graph] [ELO Gain] Test 'test_...' already exists
  → Test was already in source file
  → Skips re-adding (idempotent)
  → Proceeds with next phase
```

### ❌ Failure Indicators

Stop and investigate if you see:

```
[cody-graph] [ELO Gain] [FAIL] Could not add test: Target file not found
  → bitboard/src/lib.rs missing
  → Problem: Source code structure issue
  → Fix: Verify bitboard/ folder exists

[cody-graph] [ELO Gain] Compilation failed after adding test
  → Test has syntax errors
  → Problem: Generated test code is invalid
  → Fix: Check git diff bitboard/src/lib.rs for syntax issues

[cody-graph] [ELO Gain] [!] Unit test FAILED - issue not reproduced
  → Test compiled but failed to run
  → Problem: Test ran against working code (test is wrong)
  → Fix: Could mean engine was already fixed, try again

[commit_util] No changes to commit
  → Git sees no file modifications
  → Problem: save_test_to_source() didn't write file
  → Fix: Check file permissions on bitboard/src/lib.rs
```

## Step-by-Step Verification

### After Phase 2 (Compilation)

Check that test was actually added:

```powershell
git diff bitboard/src/lib.rs | head -100
```

Should show something like:
```diff
+#[cfg(test)]
+mod regression_tests {
+    #[test]
+    fn test_illegal_move_issue_reproduction() {
+        use crate::position::Position;
+        ...
+        assert!(!is_attacked, "Illegal move generated");
+    }
+}
```

**If diff is empty**: save_test_to_source() was not called or failed silently

### After Phase 5 (Decision/Commit)

Check git log for new commit:

```powershell
git log --oneline -5
```

Should show:
```
abc1234 Add regression test for illegal move generation
def5678 [previous commit]
...
```

Check version was bumped:

```powershell
cat engine/Cargo.toml | grep "^version"
# Should now show: version = "0.1.3" (or next patch)
```

Check binary was created:

```powershell
ls C:\chess\Engines\Cody-*.exe | Sort-Object -Descending | Select-Object -First 3
# Should show: Cody-0.1.3.exe (most recent)
```

## Running Multiple Iterations

The orchestration is designed to loop. After first success, it attempts another cycle:

```powershell
[cody-graph] [ELO Gain] [0/6] Sanity check phase  [ITERATION 2]
[cutechess] Playing 10 games...
[cody-graph] [ELO Gain] [OK] Sanity check completed: no issues
[cody-graph] [ELO Gain] [] No issues detected - Generating ELO improvement
[cody-graph] [ELO Gain] Proposed IMPROVEMENT: Better Capture Ordering...

[cody-graph] [ELO Gain] [1/6] Candidate generation phase
[ALTERNATIVE PATH]: Generates improvement proposal instead of test

[cody-graph] [ELO Gain] [2/6] Compilation & Validation
[Saves improvement diff to git]

[cody-graph] [ELO Gain] [3/6] Running Validation
[Runs 100 games against champion: gauntlet match]

[cody-graph] [ELO Gain] [4/6] Statistical check (SPRT)
[Analyzes results: +12 ELO, 95% confidence]

[cody-graph] [ELO Gain] [5/6] Decision phase
[OK] SPRT passed — ELO gain +12.0 — COMMITTING
Version: 0.1.3 → 0.1.4
```

**Loop stop condition**: Reaches `elo_successful_commits >= 5` or runs out of issues

## Post-Run Analysis

After orchestration completes, review:

### 1. Commit History
```powershell
git log --oneline -10
# Should show test commits and/or improvement commits
```

### 2. Test File Contents
```powershell
tail -50 bitboard/src/lib.rs | grep -A 30 "regression_tests"
# Should show your generated tests
```

### 3. Engine Functionality (Sanity)
```powershell
# Test the engine still works
cargo run -p engine --release < engine_stdin.txt
# Should start and accept UCI commands
```

### 4. Binary Versions Created
```powershell
ls C:\chess\Engines\Cody-*.exe | Measure-Object
# Should see new binaries created for each commit
```

## Cleanup Between Runs

If you need to reset and start over:

```powershell
# Reset to before orchestration
git reset --hard HEAD~5  # Undo last 5 commits

# Clean up test files
rm worst_fail.pgn (if exists)
rm orchestrator_state.json

# Rebuild
cargo clean
cargo build --release

# Run again
python .\cody-graph\main.py elogain
```

## Capturing Output

For analysis, save output to file:

```powershell
python .\cody-graph\main.py elogain | Tee-Object orchestration_run.log
# Output both to screen AND file

# Later review specific phases:
Select-String "Phase" orchestration_run.log
Select-String "\[OK\]" orchestration_run.log
Select-String "\[FAIL\]" orchestration_run.log
```

## Expected Outcomes

**Best case** (engine has bugs + can be fixed):
- Test added to show the bug
- Improvement proposed to fix it
- ELO gain achieved
- 2 commits: +test, +improvement
- Version: 0.1.2 → 0.1.4

**Good case** (engine has bugs, takes time to fix):
- Multiple regression tests added
- Incremental improvements
- Version rises slowly with each iteration
- Eventually reverts when stuck

**Neutral case** (engine is stable):
- Sanity check passes immediately
- ELO improvement proposals generated
- Improvements tested via gauntlet match
- Typically +5 to +15 ELO per iteration

**Edge case** (test generation fails):
- Fallback to placeholder test
- Still functional, just not position-specific
- Won't reproduce exact bug
- Safe to revert if needed
