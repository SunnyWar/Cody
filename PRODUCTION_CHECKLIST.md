# Production Readiness Checklist

## Code Quality ✅

- [x] All Python syntax validated (no errors)
- [x] Type hints present where applicable
- [x] Error handling for file I/O operations
- [x] Proper encoding (UTF-8) for source file operations
- [x] Integration points clearly defined

## Data Flow Validation ✅

**Phase 0 → Phase 1**
- [x] `sanity_result` dict created with issues
- [x] `worst_fail_pgn` field populated by sanity check
- [x] `illegal_moves` and `quick_losses` lists created

**Phase 1 → Phase 2**
- [x] `candidate` dict returned with `test_code` field
- [x] `candidate` dict includes `test_name` field
- [x] `state["elo_test_code"]` set from `candidate.get("test_code")`
- [x] `state["elo_candidate_type"] = "unit_test"` set correctly

**Phase 2 (Compilation)**
- [x] Retrieves `test_code` from state
- [x] Retrieves `test_name` from candidate
- [x] Calls `generator.save_test_to_source(test_code, test_name)`
- [x] Handles success/failure cases
- [x] Proceeds to Phase 3 on success

**Phase 3 → Phase 5 (Validation → Decision)**
- [x] Test runs and result captured
- [x] Decision phase checks `test_result == "PASS"`
- [x] Commit happens on PASS
- [x] Version bumped correctly
- [x] Binary copied with new version

## Edge Cases Handled ✅

- [x] No test code returned → early exit in Phase 2
- [x] File not found during save → error returned
- [x] Test already exists → skipped gracefully
- [x] Compilation fails after adding test → full revert
- [x] Test runs but fails → goes to revert phase
- [x] Empty worst_fail.pgn → falls back to placeholder test

## Known Behaviors

### What Happens If...

**Sanity check finds no issues?**
- `has_critical = False`, `has_warnings = False`, `has_quick_losses = False`
- Goes to improvement generation instead
- Proceeds with ELO gain proposal (not test)
- ✅ This is correct behavior

**worst_fail.pgn doesn't exist?**
- `parse_worst_fail_pgn()` returns empty list
- Falls back to `_placeholder_unit_test()` 
- Still generates a test, but generic instead of position-specific
- ✅ Graceful degradation

**Multiple failing games in worst_fail.pgn?**
- `parse_worst_fail_pgn()` returns list of all games
- `_generate_position_specific_unit_test()` uses `failing_games[0]` (first one)
- Each orchestration run picks first failing game
- Next run finds other failing games
- ✅ Sequential fixing over multiple iterations

**Test code is very long?**
- No size limits enforced
- If test > 10KB, insertion still works
- May look odd in bitboard/src/lib.rs but functionally OK
- ✅ Acceptable tradeoff

## File Modification Safety ✅

**bitboard/src/lib.rs (target of save_test_to_source)**
- [x] Only appended to (no removal/truncation)
- [x] Creates `#[cfg(test)] mod regression_tests` if missing
- [x] Uses standard Rust test syntax
- [x] Can be git staged/committed safely
- [x] Can be reverted if bad

**orchestrator_state.json**
- [x] Created/updated by phase handlers
- [x] Survives between runs if needed
- [x] Can be manually inspected for debugging

## Retry Safety ✅

If orchestration interrupted and re-run:
- [x] Test already in source → skipped (won't add twice)
- [x] Git already committed → git add shows nothing new
- [x] No duplicate compilation attempts
- [x] State preserved across runs (if file persists)

## Output Clarity ✅

Messages logged at each step:
- [x] Test name being added
- [x] File path where test added
- [x] Success/failure of file write
- [x] Compilation result
- [x] Test execution result
- [x] Commit success/failure
- [x] Version numbers before/after

All messages use `[cody-graph]` prefix for consistencyand `[OK]` for success, `[FAIL]` for errors.

## Success Verification Steps

After running orchestration, verify:

1. **Check git status**
   ```powershell
   git status
   ```
   Should show: `bitboard/src/lib.rs` modified

2. **Verify test in source**
   ```powershell
   git diff bitboard/src/lib.rs | head -50
   ```
   Should show test code added in regression_tests module

3. **Run test manually**
   ```powershell
   cargo test test_illegal_move_issue_reproduction -- --nocapture
   ```
   Should show: `test ... ok`

4. **Check version bump**
   ```powershell
   cat engine/Cargo.toml | grep "^version"
   ```
   Version should have incremented

5. **Verify binary created**
   ```powershell
   ls C:\chess\Engines\Cody*.exe | Sort-Object -Descending | Select-Object -First 1
   ```
   Should show newest binary with version number

## Zero-Risk Rollback

If test causes issues:
```powershell
git revert HEAD  # Reverts test commit
git reset --hard origin/main  # Full rollback (if needed)
```

Since test is only test code (not engine changes), no functional impact even if test is wrong.
