# Clippy Execution Prompt

You are a senior Rust engineer fixing a specific clippy warning for the Cody chess engine.

## Your Task

Implement the following clippy fix:

**{CLIPPY_DETAILS}**

## Requirements

1. **Maintain Correctness**
   - No behavior changes beyond the lint fix
   - All tests must pass
   - Perft results must remain identical

2. **Performance and Memory Focus**
   - Prefer fixes that remove allocations or reduce copies
   - Avoid changes that slow down hot paths
   - Keep changes minimal and targeted

3. **Code Quality**
   - Follow existing patterns and style
   - Add brief comments only when necessary
   - Avoid unnecessary refactors unrelated to the lint

4. **Validation**
   - Ensure `cargo clippy --all-targets --all-features -- -W clippy::perf` passes
   - Ensure `cargo test` passes

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
- Keep the patch focused on the clippy fix only
