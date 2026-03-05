# Quick Reference Card

## Command to Run

```powershell
cd D:\Cody
python .\cody-graph\main.py elogain
```

## What Happens (6 Phases)

| Phase | Does What | Success Output |
|-------|-----------|-----------------|
| 0 | Plays 10 games, detects bugs | `[OK] Extracted worst_fail.pgn with N games` |
| 1 | Generates test from failing position | `Analyzed N failing games` + test name |
| 2 | Adds test to source, compiles | `[OK] Test ... added to bitboard/src/lib.rs` |
| 3 | Runs the test you just added | `[OK] Unit test PASSED` |
| 4 | (Skipped for unit tests) | — |
| 5 | Commits to git, bumps version | `Version: X.X.X → X.X.Y` |

**Total time**: 5-20 minutes

## Critical Success Indicators

✅ **All of these should appear**:
1. `[OK] Extracted worst_fail.pgn`
2. `[OK] Test ... added to bitboard/src/lib.rs`
3. `[OK] Unit test PASSED`
4. `Version: X → Y`
5. `Copied to C:\chess\Engines\...`

## What to Check After Running

```powershell
# 1. See what changed
git diff --stat

# 2. Look at the actual test added
git diff bitboard/src/lib.rs | head -100

# 3. Verify test works manually
cargo test test_illegal_move_issue_reproduction

# 4. Check version/commit
git log -1
git show --stat
```

## If Something Goes Wrong

| Error Message | Likely Cause | Check This |
|---------------|--------------|-----------|
| `Target file not found` | bitboard/src/lib.rs missing | `ls bitboard/src/lib.rs` |
| `Compilation failed` | Bad test syntax | `git diff bitboard/src/lib.rs` |
| `Unit test FAILED` | Test doesn't reproduce bug | Could mean code was fixed |
| `No changes to commit` | File not actually written | Check file permissions |
| `No worst_fail.pgn` | Sanity check found no issues | Good! Engine is stable |

## Expected File Changes

After successful run:
```
bitboard/src/lib.rs          → test added
engine/Cargo.toml            → version bumped
.git/objects/                → new commit
C:\chess\Engines\Cody-*.exe  → new binary
```

## Keep These Commands Handy

```powershell
# See current changes
git status

# See what was added
git diff bitboard/src/lib.rs

# See last few commits
git log -5 --oneline

# Run the test manually
cargo test test_illegal_move_issue_reproduction -- --nocapture

# Switch back if needed (undo last N commits)
git reset --hard HEAD~1

# Check versions
cat engine/Cargo.toml | grep version

# List binaries created
ls C:\chess\Engines\Cody-*.exe | wc -l
```

## Success Email Template

After orchestration runs successfully:

> Orchestration completed successfully!
>
> **Summary**:
> - Sanity check: ✅ Issues found
> - Test generated: ✅ From worst_fail.pgn
> - Test committed: ✅ Version bumped X → Y
>
> **Verification**:
> ```
> git log -1
> git diff HEAD~1 bitboard/src/lib.rs | head -50
> cargo test test_illegal_move_issue_reproduction
> ```
>
> **Next step**: [Choose from below]
> - [ ] Run again to find more issues
> - [ ] Inspect test in source to understand fix needed
> - [ ] Manually implement the fix
> - [ ] Stop and analyze patterns

---

## Documentation Files Created

- `ORCHESTRATION_FIX_SUMMARY.md` → What was fixed and why
- `PRODUCTION_CHECKLIST.md` → All safety checks pass
- `EXECUTION_GUIDE.md` → Detailed step-by-step walkthrough
- `CODE_CHANGES.md` → Exact code changes made
- `QUICK_REFERENCE_CARD.md` → This file!

Read them in order for full understanding, or use this card for quick lookup.
