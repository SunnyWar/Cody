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

Provide your implementation as a unified diff patch that can be applied with `git apply`:

```diff
diff --git a/path/to/file.rs b/path/to/file.rs
index abc123..def456 100644
--- a/path/to/file.rs
+++ b/path/to/file.rs
@@ -10,7 +10,7 @@ old line
-removed line
+added line
 unchanged line
```

**Important**: 
- Use exact file paths from repo root
- Include proper git diff headers
- Ensure the patch is self-contained and applies cleanly
- Test the refactoring mentally before generating the patch

## Validation Checklist

Before submitting your patch, verify:
- [ ] No new heap allocations in hot paths
- [ ] All public API changes are backward compatible (or breaking changes are clearly marked)
- [ ] Changes respect separation between bitboard and engine crates
- [ ] Code follows existing patterns and conventions
- [ ] Changes can be tested with existing test suite
