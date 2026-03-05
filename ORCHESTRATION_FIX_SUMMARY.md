# ELO Gain Orchestration Fix Summary

## Issues Fixed

### 1. **Quick Losses Not Triggering Test Mode**
- **Problem**: Only illegal moves triggered unit test generation; quick losses (checkmate in 0 moves) were categorized as warnings and skipped
- **Fix**: Modified `elo_gain_candidate_generation()` to check `if has_critical or has_warnings or has_quick_losses`
- **Impact**: Now quick losses will trigger position-specific test generation

### 2. **Test Code Not Persisting to Source Files**
- **Problem**: Tests were generated and ran successfully but were never added to source files, causing them to disappear after orchestration ended
- **Fix**: 
  - Added `save_test_to_source()` method to `CandidateGenerator` class
  - Updated compilation phase to call this method with test code and test name
  - Tests are now written to `bitboard/src/lib.rs` before compilation
- **Impact**: Tests persist in git and become regression tests for future runs

### 3. **Incorrect Issue Category Logic**
- **Problem**: Both illegal moves AND quick losses now trigger path to `_generate_position_specific_unit_test()`
- **Fix**: Consolidated logic to analyze actual failing positions from `worst_fail.pgn` for both issue types
- **Impact**: Tests are now based on real failing game positions, not generic templates

## Expected Execution Flow

When you run `python .\cody-graph\main.py elogain`:

### Phase 0: Sanity Check
```
[10 self-play games] 
   → Detects illegal moves or quick losses
   → Extracts problematic games to worst_fail.pgn
   → Returns sanity_result with issues
```

### Phase 1: Candidate Generation ⭐ KEY CHANGES HERE
```
If issues found:
   1. Call parse_worst_fail_pgn() → Gets list of failing games with FENs
   2. Call infer_bug_pattern() → Analyzes error type
   3. Call _generate_position_specific_unit_test() → Creates test from actual position
   
Returns:
   {
     "test_type": "unit",
     "test_name": "test_illegal_move_issue_reproduction",
     "test_code": "[Rust test code using exact FEN from failing game]",
     "title": "Position-Specific Illegal Move Test",
     ...
   }
```

### Phase 2: Compilation & Validation ⭐ KEY CHANGES HERE
```
If unit test mode:
   1. Extract test_code and test_name from candidate
   2. Call save_test_to_source(test_code, test_name)
      → Adds test to bitboard/src/lib.rs
      → Creates #[cfg(test)] mod regression_tests if needed
   3. cargo build to verify it compiles
   
If successful:
   [Test code is now in source file]
```

### Phase 3: Validation
```
If unit test mode:
   1. cargo test [--nocapture] to run the test
   2. Test verifies the position doesn't generate illegal moves
   
Test should PASS because:
   - Uses exact position from a game that WAS broken
   - Tests Rust `generate_pseudo_moves()` API directly
   - Once fixed, test stays in codebase as regression test
```

### Phase 4: Statistical Check
```
Just passes through for unit test mode
```

### Phase 5: Decision
```
If unit test passed:
   1. Git commit with message "Add regression test for illegal move generation"
   2. Version bump: 0.1.2 → 0.1.3 (patch version)
   3. Binary copied with new version
   
state["elo_improvement_committed"] = "test_added"
state["elo_successful_commits"] = 1
```

## What Success Looks Like

Expected terminal output:
```
[cody-graph] [ELO Gain] [0/6] Sanity check phase
[cutechess] Playing 10 games...
[cody-graph] [ELO Gain] [OK] Sanity check completed: warnings found
[cody-graph] [ELO Gain] [OK] Extracted worst_fail.pgn with 3 problematic games

[cody-graph] [ELO Gain] [1/6] Candidate generation phase
[candidate_generator] Analyzed 3 failing games
[candidate_generator] Inferred bug pattern: illegal_move_generation
[cody-graph] [ELO Gain] Proposed TEST: Position-Specific Illegal Move Test
  Description: Tests the exact position where illegal move was generated...
  Test function: test_illegal_move_issue_reproduction

[cody-graph] [ELO Gain] [2/6] Compilation & Validation phase
[cody-graph] [ELO Gain] [OK] Test 'test_illegal_move_issue_reproduction' added to bitboard/src/lib.rs
[cody-graph] [ELO Gain] [OK] Test code compiles successfully

[cody-graph] [ELO Gain] [3/6] Running Validation
[cody-graph] [ELO Gain] [OK] Unit test PASSED - issue successfully reproduced!

[cody-graph] [ELO Gain] [5/6] Decision phase
[cody-graph] [ELO Gain] [OK] Unit test successfully reproduces issue — COMMITTING
[commit_util] Added regression test for illegal move generation
[cody-graph] [ELO Gain] Version: 0.1.2 → 0.1.3
```

**Key verification**: After orchestration completes, check `bitboard/src/lib.rs` and look for:
```rust
#[cfg(test)]
mod regression_tests {
    #[test]
    fn test_illegal_move_issue_reproduction() {
        // Test code here...
    }
}
```

## What to Debug If It Fails

### Test Not Added to Source File
Check:
```powershell
git diff bitboard/src/lib.rs
# Should show test being added
```

**If no diff**: 
- Check `[cody-graph] [ELO Gain] [OK] Test ... added` message in output
- If missing: `save_test_to_source()` may not be called
- Look for "[FAIL] Could not add test" message

### Test Compilation Fails
Check terminal for:
```
error[E0433]: cannot find function `generate_pseudo_moves`
```

**If you see this**:
- Test code has wrong imports
- Check bitboard/src/lib.rs to see what was actually added
- Verify `use crate::movegen::api::generate_pseudo_moves;` is correct

### Test Execution Fails
Check:
```powershell
cargo test test_illegal_move_issue_reproduction -- --nocapture
```

**Expected**: Test should PASS (meaning the position was indeed problematic before)
**If FAIL**: The FEN from worst_fail.pgn may not be accurate, or the position doesn't actually trigger the bug

### worst_fail.pgn Not Created
Check:
```powershell
ls -la worst_fail.pgn
```

**If missing**:
- Sanity check may not have found issues
- Look for "Extracted worst_fail.pgn" message in Phase 0 output
- If missing: games weren't classified as problematic

## Files Modified

1. **cody-graph/elo_tools/candidate_generator.py**
   - Enhanced `generate_unit_test_for_issue()` to trigger on warnings/quick_losses too
   - Changed to always call `parse_worst_fail_pgn()` and `infer_bug_pattern()`
   - Added `save_test_to_source()` to actually write test code to files

2. **cody-graph/agents/elo_gain_agent.py**
   - Updated `elo_gain_candidate_generation()` to check for warnings and quick_losses
   - Updated `elo_gain_compilation_check()` to call `save_test_to_source()`
   - Changed phase_stage to `"unit_test"` for proper routing

## Testing Locally

Run:
```powershell
cd D:\Cody
python .\cody-graph\main.py elogain
```

Monitor output for:
1. "Extracted worst_fail.pgn" - Files found with problems
2. "Analyzed X failing games" - Positions parsed
3. "Test ... added to bitboard/src/lib.rs" - File modified
4. "Unit test PASSED" - Test ran successfully
5. "Version: X → Y" - Commit successful

## One More Thing

The system will now loop! After the first test is committed, `elo_successful_commits` increments. If you want multiple fixes in sequence, the orchestration can continue to:
1. Run sanity check again
2. Find new issues (if any)
3. Generate next test
4. Repeat

Stop condition: `elo_successful_commits >= 5` (configurable in Phase 5 decision logic)
