# Refactoring Execution Prompt

You are a senior Rust engineer implementing a specific refactoring for the Cody chess engine.

## Your Task

Implement the following refactoring:

**{REFACTORING_DETAILS}**

## Requirements

1. **Maintain Correctness**
   - All existing tests must pass after refactoring
   - Perft results must remain identical
   - No behavior changes—pure refactoring only

2. **Preserve Performance**
   - No allocations added to hot paths (move gen, search, position updates)
   - Benchmark critical paths if changes might affect them
   - Keep the fixed-block arena architecture intact

3. **Code Quality Standards**
   - Follow existing code style and patterns
   - Add doc comments for any new public APIs
   - Use idiomatic Rust patterns
   - Maintain type safety with strong newtypes

4. **Testing Requirements**
   - Run `cargo test -p bitboard` for bitboard changes
   - Run `cargo test -p engine` for engine changes
   - Run `cargo test` for workspace-wide changes
   - Verify with `cargo build --release`

5. **Documentation**
   - Update inline comments if logic changes
   - Update module-level docs if structure changes
   - Add examples for new public APIs

## Output Format

Provide the complete updated file content:

```rust
// Complete file content here
// path/to/file.rs

[full file content with your changes]
```

**CRITICAL**: 
- Output the COMPLETE file with all changes applied
- Include the file path as a comment at the top
- Use proper Rust syntax
- Preserve all existing functionality not being changed
- Test the refactoring mentally before generating the code

## Validation Checklist

Before submitting your patch, verify:
- [ ] No new heap allocations in hot paths
- [ ] All public API changes are backward compatible (or breaking changes are clearly marked)
- [ ] Changes respect separation between bitboard and engine crates
- [ ] Code follows existing patterns and conventions
- [ ] Changes can be tested with existing test suite
