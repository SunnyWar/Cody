# Code Changes Summary

## File 1: cody-graph/elo_tools/candidate_generator.py

### Change 1: Enhanced threshold for unit test generation

**Location**: `generate_unit_test_for_issue()` method (around line 310)

**Before**:
```python
if illegal_moves or quick_losses:
    # Branch only for illegal moves
    if illegal_moves:
        # Generate test for illegal moves
    elif quick_losses:
        # This was dead code
```

**After**:
```python
if illegal_moves or quick_losses:
    # Both illegal moves and quick losses trigger position-specific testing
    if illegal_moves:
        test_focus = "reproduce_illegal_move"
        test_variant = "unit"
        issue_description = f"Illegal moves found: {len(illegal_moves)} occurrences"
    else:
        test_focus = "reproduce_bad_evaluation"
        test_variant = "unit"  # Treat as unit test, not integration
        issue_description = f"Quick losses found: {len(quick_losses)} occurrences"
    
    # ALWAYS TRY TO ANALYZE ACTUAL FAILING POSITIONS
    failing_games = self.parse_worst_fail_pgn()
    if failing_games:
        bug_pattern = self.infer_bug_pattern(failing_games)
        print(f"[candidate_generator] Analyzed {len(failing_games)} failing games")
        print(f"[candidate_generator] Inferred bug pattern: {bug_pattern.get('bug_type', 'unknown')}")
        return self._generate_position_specific_unit_test(failing_games, bug_pattern)
```

**Impact**: Quick losses now trigger position-specific tests instead of being skipped

### Change 2: Added `save_test_to_source()` method

**Location**: After `_placeholder_improvement()` method (around line 745)

**New Code**:
```python
def save_test_to_source(self, test_code: str, test_name: str) -> tuple[bool, str]:
    """
    Save generated test code to the appropriate source file.
    Returns (success, message).
    """
    # Determine target file based on test type
    test_target = self.repo_path / "bitboard" / "src" / "lib.rs"
    
    if not test_target.exists():
        return False, f"Target file not found: {test_target}"
    
    try:
        content = test_target.read_text(encoding='utf-8')
        
        # Check if test already exists (via test name/module)
        if f"mod {test_name}" in content or f"fn {test_name}" in content:
            return False, f"Test '{test_name}' already exists in {test_target}"
        
        # Find insertion point - add before the last few closing braces
        lines = content.split('\n')
        insertion_line = len(lines) - 1
        
        # Try to find a test module section
        for i in range(len(lines) - 1, max(len(lines) - 20, 0), -1):
            if "#[cfg(test)]" in lines[i]:
                insertion_line = i + 1
                while insertion_line < len(lines) and lines[insertion_line].strip():
                    insertion_line += 1
                break
        
        # If no test section exists, add one before final closing brace
        if insertion_line == len(lines) - 1:
            lines.insert(insertion_line, '')
            lines.insert(insertion_line + 1, '#[cfg(test)]')
            lines.insert(insertion_line + 2, 'mod regression_tests {')
            insertion_line = insertion_line + 3
            
            # Insert the test code
            lines.insert(insertion_line, test_code)
            lines.insert(insertion_line + 1, '}')
        else:
            # Insert into existing test section
            lines.insert(insertion_line, test_code)
        
        modified_content = '\n'.join(lines)
        
        # Write back
        test_target.write_text(modified_content, encoding='utf-8')
        return True, f"Test '{test_name}' added to {test_target}"
        
    except Exception as e:
        return False, f"Failed to write test: {str(e)}"
```

**Impact**: Tests can now be persisted to source files

---

## File 2: cody-graph/agents/elo_gain_agent.py

### Change 1: Enhanced issue detection in candidate generation

**Location**: `elo_gain_candidate_generation()` function (around line 160)

**Before**:
```python
has_critical = sanity_result.get("has_critical_issues", False)
has_warnings = len(sanity_result.get("warnings", [])) > 0

if has_critical or has_warnings:
```

**After**:
```python
# Check if sanity check found issues
# NOTE: Both critical issues AND warnings should trigger test generation
# because quick losses (checkmate in 0 moves) are also serious bugs
has_critical = sanity_result.get("has_critical_issues", False)
has_warnings = len(sanity_result.get("warnings", [])) > 0
has_quick_losses = len(sanity_result.get("quick_losses", [])) > 0

if has_critical or has_warnings or has_quick_losses:
```

**Impact**: Quick losses now treated as triggering unit test generation

### Change 2: Actually save test code in compilation phase

**Location**: `elo_gain_compilation_check()` function (around line 266)

**Before**:
```python
if candidate_type == "unit_test":
    print(f"[cody-graph] [ELO Gain] [] Adding unit test: {candidate.get('function_name', 'unknown')}")
    
    test_code = state.get("elo_test_code", "")
    test_files = state.get("elo_test_files", [])
    
    if not test_code:
        print("[cody-graph] [ELO Gain] [] No test code to add", flush=True)
        state["status"] = "compilation_failed"
        state["elo_phase_stage"] = "revert"
        return state
    
    # TODO: Actually add test code to files
    print("[cody-graph] [ELO Gain] [] [TODO] Test code would be added to: ...", flush=True)
    
    # Verify core engine still compiles
    compilation_ok = validate_compilation(...)
```

**After**:
```python
if candidate_type == "unit_test":
    print(f"[cody-graph] [ELO Gain] [] Adding unit test: {candidate.get('test_name', 'unknown')}")
    
    test_code = state.get("elo_test_code", "")
    test_name = candidate.get("test_name", "regression_test")
    
    if not test_code:
        print("[cody-graph] [ELO Gain] [] No test code to add", flush=True)
        state["status"] = "compilation_failed"
        state["elo_phase_stage"] = "revert"
        return state
    
    # ACTUALLY ADD TEST CODE TO SOURCE FILE
    generator = CandidateGenerator(repo_path)
    success, message = generator.save_test_to_source(test_code, test_name)
    
    if not success:
        print(f"[cody-graph] [ELO Gain] [FAIL] Could not add test: {message}", flush=True)
        state["status"] = "compilation_failed"
        state["elo_phase_stage"] = "revert"
        return state
    
    print(f"[cody-graph] [ELO Gain] [OK] {message}", flush=True)
    
    # Verify core engine still compiles with new test
    compilation_ok = validate_compilation(...)
    
    if not compilation_ok:
        print("[cody-graph] [ELO Gain] Compilation failed after adding test, reverting", flush=True)
        state["status"] = "compilation_failed"
        state["elo_phase_stage"] = "revert"
        return state
    
    print("[cody-graph] [ELO Gain] [OK] Test code compiles successfully", flush=True)
    state["status"] = "ok"
    state["elo_phase_stage"] = "unit_test"  # Go to unit test validation
```

**Impact**: 
- Tests are now written to source files with `save_test_to_source()`
- Proper error handling if write fails
- Phase routing changed to `"unit_test"` for proper validation

### Change 3: Updated imports

**Location**: Top of `elo_gain_compilation_check()` (around line 260)

**Before**:
```python
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "elo_tools"))
from validate_compilation import validate_compilation
```

**After**:
```python
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "elo_tools"))
from validate_compilation import validate_compilation
from candidate_generator import CandidateGenerator
```

**Impact**: `CandidateGenerator` now imported for use in compilation phase

---

## Unchanged But Important

### No changes to:
- Sanity check (still detects issues)
- Worst_fail.pgn extraction (still saves failing games)
- Parse_worst_fail_pgn() (still reads PGN)
- Infer_bug_pattern() (still analyzes bugs)
- _generate_position_specific_unit_test() (still creates specific tests)
- Decision phase logic (still commits on test PASS)
- Version bumping (still increments patch version)

## Key Integration Points

**Flow After Fixes**:
```
Sanity Check
    ↓ (detects illegal_moves OR quick_losses OR warnings)
Candidate Generation
    ├─ has_critical OR has_warnings OR has_quick_losses = TRUE
    ├─ Call parse_worst_fail_pgn() ← Existing code, now always called
    ├─ Call infer_bug_pattern() ← Existing code, now always called
    └─ Return _generate_position_specific_unit_test() ← Existing code, now always used
Compilation & Validation
    ├─ NEW: Call save_test_to_source(test_code, test_name)
    ├─ NEW: Write test to bitboard/src/lib.rs
    ├─ NEW: Verify it compiles with cargo build
    └─ Proceed to unit test validation
Unit Test Validation
    ├─ Run: cargo test [test_name]
    ├─ Expect: PASS (test reproduces the issue)
    └─ Proceed to decision
Decision
    ├─ Test PASSED? → Commit
    ├─ Update git history
    ├─ Bump version
    └─ Copy binary with new version
```

## Testing Changes Locally

Before running full orchestration, you can verify:

```powershell
# 1. Test the save_test_to_source method
python -c "
from cody_graph.elo_tools.candidate_generator import CandidateGenerator
gen = CandidateGenerator('D:\\Cody')
test_code = '#[test]\nfn test_example() { assert!(true); }'
success, msg = gen.save_test_to_source(test_code, 'test_example')
print(f'Success: {success}')
print(f'Message: {msg}')
"

# 2. Check the enhanced method was triggered
python -c "
from cody_graph.elo_tools.candidate_generator import CandidateGenerator
gen = CandidateGenerator('D:\\Cody')
sanity = {
    'quick_losses': [1, 2, 3],  # Has issues
    'illegal_moves': [],
    'warnings': [],
    'worst_fail_pgn': 'worst_fail.pgn'
}
result = gen.generate_unit_test_for_issue(sanity)
print(f'Test name: {result.get(\"test_name\")}')
print(f'Has test_code: {bool(result.get(\"test_code\"))}')
"
```

---

## Summary

| Component | Before | After | Impact |
|-----------|--------|-------|--------|
| Quick loss handling | Warnings, skipped | Critical, trigger test | Tests now created for quick losses |
| Test persistence | Generated in memory, lost | Saved to bitboard/src/lib.rs | Tests survive beyond orchestration |
| Issue analysis | Generic templates | Position-specific from worst_fail.pgn | Tests reproduce actual bugs |
| File I/O | Not implemented | Full implementation with error handling | Safe writing with validation |
| Regression test suite | None | Grows with each successful test | Long-term engine robustness |

All changes are **backward compatible** and **non-breaking**.
