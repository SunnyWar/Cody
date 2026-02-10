# World-Class Feature Execution Prompt

You are a chess engine expert implementing a specific feature for the Cody chess engine.

## Your Task

Implement the following feature:

**{FEATURE_DETAILS}**

## Requirements

1. **Correctness First**
   - Implement the feature correctly according to chess engine theory
   - All existing tests must pass
   - Add new tests for the feature
   - Verify with perft if move generation is affected
   - No regressions in existing functionality

2. **Architecture Compliance**
   - Respect the fixed-block arena architecture
   - Maintain allocation-free hot paths
   - Follow separation between bitboard and engine crates
   - Use existing patterns and conventions
   - Preserve type safety with newtypes

3. **Performance Awareness**
   - Profile the feature's impact on search speed
   - Measure ELO impact if possible (via self-play)
   - Optimize hot paths in the implementation
   - Use benchmarks to verify performance
   - Document any performance trade-offs

4. **Code Quality**
   - Write idiomatic Rust
   - Add comprehensive doc comments
   - Include usage examples for new APIs
   - Follow existing naming conventions
   - Add inline comments for complex logic

5. **Testing Requirements**
   - Add unit tests for the feature
   - Add integration tests if needed
   - Test edge cases thoroughly
   - Run full test suite: `cargo test`
   - Build in release: `cargo build --release`
   - Run benchmarks: `cargo bench -p engine`

6. **Documentation**
   - Update relevant .md files (architecture.md, etc.)
   - Document the feature's purpose and usage
   - Add references to chess programming theory
   - Document configuration/tuning parameters
   - Include examples of expected behavior

## Implementation Best Practices

### For Search Features
- Integrate cleanly with existing negamax/alpha-beta
- Consider impact on transposition table
- Add proper move ordering integration
- Include depth/ply handling correctly
- Add debug counters for analysis

### For Evaluation Features
- Make evaluation incremental if possible
- Consider both opening and endgame phases
- Add tunable parameters with sensible defaults
- Preserve evaluation symmetry
- Add tests with known positions

### For UCI Features
- Follow UCI protocol specification exactly
- Handle all edge cases (invalid input, etc.)
- Add proper error messages
- Test with real UCI GUI if possible
- Document new options/commands

## Output Format

Provide the complete updated file content:

```rust
// path/to/file.rs

[complete file content with feature implemented]
```

**CRITICAL**:
- Output the COMPLETE file with all changes applied
- Include the file path as a comment at the top
- Implement the feature fully and correctly

**Include**:
- Main feature implementation
- Tests for the feature
- Documentation updates
- Any necessary configuration

## Validation Checklist

Before submitting:
- [ ] Feature implemented according to chess theory
- [ ] All tests pass: `cargo test`
- [ ] New tests added for feature
- [ ] Release build succeeds: `cargo build --release`
- [ ] Benchmarks run cleanly: `cargo bench -p engine`
- [ ] No allocations in hot paths
- [ ] Documentation updated
- [ ] Code follows project conventions
- [ ] ELO impact measured or estimated
- [ ] No regressions in existing functionality

## ELO Impact Measurement

If possible, measure ELO impact via:
1. Self-play: engine with feature vs. without (100+ games)
2. Test suite: tactical test positions (WAC, etc.)
3. Perft performance: ensure no slowdown
4. Search statistics: nodes/second, etc.

Document the measurement methodology and results.

## Safety & Soundness

- Ensure all chess rules are correctly implemented
- Verify no illegal moves are generated/played
- Check for integer overflow in scores
- Validate all input from UCI commands
- Handle all error cases gracefully
