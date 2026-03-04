# Dual-Mode Test Generation System

## Overview
The candidate generator now intelligently chooses between **unit tests** and **integration tests** based on the type of issue detected by the sanity check phase.

## Test Selection Logic

### Unit Tests (Isolated Issues)
**When to use:** When the issue is localized to move generation or position evaluation
- **Illegal moves**: Move generation bugs → `test_all_moves_are_legal()`
- **Single-position eval issues**: Evaluation bugs → `test_evaluation_differences()`
- **General crashes**: Stability issues → `test_position_apply_safety()`

**Characteristics:**
- Run quickly (< 100ms)
- Test isolated modules (moveGen, evaluation, position ops)
- Self-contained, no game context needed
- Module: `bitboard` or `engine` (depending on scope)

### Integration Tests (Game-Level Issues)
**When to use:** When the issue affects game-level behavior across multiple moves
- **Quick losses**: Search/evaluation failures → `test_game_search_quality()`
- **Game stability**: Multiple move sequences → `test_self_play_stability()`

**Characteristics:**
- Slower (1-5 seconds) but tests realistic game scenarios
- Verify engine behavior across multiple moves/turns
- Can detect issues that only surface during play
- Module: `engine` (requires search engine)

## Implementation

### In `candidate_generator.py`:

1. **`generate_unit_test_for_issue()`** — Main dispatcher
   - Analyzes sanity check results
   - Determines test type based on issue categorization
   - Routes to appropriate generator

2. **`_generate_unit_test()`** — Unit test generator
   - Prompts LLM for isolated test code
   - Falls back to placeholder templates
   - Used for move generation bugs

3. **`_generate_integration_test()`** — Integration test generator
   - Prompts LLM for multi-move test code
   - Falls back to placeholder templates
   - Used for game-level issues

4. **`_placeholder_unit_test()`** — Unit test fallbacks
   - Illegal Move Detection
   - Evaluation Sanity Check
   - Position Safety Check

5. **`_placeholder_integration_test()`** — Integration test fallbacks
   - Search Quality in Game
   - Self-Play Stability

## Integration with ELO Workflow

### Phase 1: Candidate Generation
```
Sanity Check Result
    ↓
Has Issues? (illegal_moves or quick_losses)
    ├─ YES → generate_unit_test_for_issue()
    │         ├─ Illegal moves → Unit test
    │         └─ Quick losses → Integration test
    │
    └─ NO → generate_improvement_proposal()
            (normal ELO improvement mode)
```

### Phase 2-3: Compilation & Validation
- **Unit test mode**: Compile test code, verify it builds
- **Integration test mode**: Add test to test suite, run cargo test
- **Improvement mode**: Validate improvement compiles normally

### Phase 4-5: Statistics & Decision
- **Unit test mode**: Track test validation (fixed = commit, still broken = iterate)
- **Integration test mode**: Same as unit tests
- **Improvement mode**: Evaluate ELO gain via gauntlet

## Example Workflow

### Scenario 1: Illegal Move Bug
```
Sanity Check → Finds illegal moves
↓
Candidate Gen → Detects "illegal_moves" issue
↓
Chooses → UNIT TEST (test_all_moves_are_legal)
↓
Compilation → Verifies test code compiles with engine
↓
Validation → Runs cargo test - if test reproduces issue, captures in repository
↓
Decision → Commit test as regression prevention
```

### Scenario 2: Poor Search Decisions
```
Sanity Check → Finds quick checkmate losses
↓
Candidate Gen → Detects "quick_losses" issue
↓
Chooses → INTEGRATION TEST (test_game_search_quality)
↓
Compilation → Adds test to test suite, verifies compiles
↓
Validation → Runs cargo test with integration test
↓
Decision → If test reproduces issue, commit as regression prevention
```

### Scenario 3: Engine is Stable
```
Sanity Check → Finds no critical issues
↓
Candidate Gen → Detects no issues
↓
Chooses → IMPROVEMENT mode (normal ELO improvement)
↓
Generates → LLM-based improvement proposal (e.g., MVV-LVA)
↓
Proceeds → Normal ELO testing workflow
```

## Benefits

1. **Correctness First**: Bugs are reproduced with tests before attempting fixes
2. **Appropriate Granularity**: Unit tests for isolated bugs, integration tests for game-level issues
3. **Regression Prevention**: Generated tests become permanent test suite additions
4. **Reduced False Positives**: Only attempt improvements when engine is behaviorally sound
5. **Faster Feedback**: Unit tests run in ~100ms, integration tests in ~5s (vs 30+ minute gauntlets)

## LLM Prompt Strategy

### Unit Test Prompts
- Focus on specific chess positions
- Ask for minimal reproducible example
- Target: fast execution, clear failure condition

### Integration Test Prompts
- Focus on game sequence behavior
- Allow multiple moves for diagnosis
- Target: realistic scenarios, game-level correctness

Both include fallback templates that are always valid Rust code.
