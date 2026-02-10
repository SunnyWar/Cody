# Performance Execution Prompt

You are a performance optimization expert implementing a specific optimization for the Cody chess engine.

## Your Task

Implement the following performance optimization:

**{PERFORMANCE_DETAILS}**

## Requirements

1. **Maintain Correctness**
   - All tests must pass: `cargo test`
   - Perft results must be identical
   - No logic bugs introduced
   - Chess rules must remain exact

2. **Measure Performance**
   - Run relevant benchmarks before and after: `cargo bench -p engine`
   - Document the actual speedup achieved
   - If < 5% improvement, reconsider the change
   - Profile critical sections if possible

3. **Code Quality**
   - Document any `unsafe` blocks thoroughly
   - Add inline comments for non-obvious optimizations
   - Keep code maintainable despite optimization
   - Use clear variable names even in hot paths

4. **Testing Requirements**
   - Run full test suite: `cargo test`
   - Run release build: `cargo build --release`
   - Run benchmarks: `cargo bench -p engine`
   - Verify perft: `cargo run --release -p engine -- perft 5`

5. **Optimization Techniques**
   - Prefer algorithmic improvements over micro-optimizations
   - Use profiling to guide decisions
   - Consider compiler optimization hints (`#[inline]`, `#[cold]`, etc.)
   - Leverage Rust zero-cost abstractions
   - Add `const` where applicable

6. **Documentation**
   - Explain the optimization in comments
   - Document performance impact
   - Note any safety considerations
   - Add examples if API changes

## Output Format

Provide two things:

### 1. Complete Updated File

```rust
// path/to/file.rs

[complete file content with optimizations applied]
```

**CRITICAL**:
- Output the COMPLETE file with all changes applied
- Include the file path as a comment at the top
- Preserve all existing functionality

### 2. Performance Report

```markdown
## Performance Impact

**Benchmark**: [name of benchmark or operation]
**Before**: [timing/throughput]
**After**: [timing/throughput]
**Improvement**: [X% faster / Yx speedup]

**Method**: [brief description of how measured]
**Test command**: `cargo bench -p engine -- [pattern]`
```

## Validation Checklist

Before submitting:
- [ ] All tests pass (`cargo test`)
- [ ] Performance measured with benchmarks
- [ ] Actual speedup >= 5% (or justified architectural improvement)
- [ ] No allocations added to hot paths
- [ ] Any `unsafe` code has detailed safety comments
- [ ] Perft results unchanged
- [ ] Code remains readable and maintainable

## Safety Requirements for `unsafe`

If using `unsafe`, document:
1. **Why it's needed**: What safe alternative was insufficient
2. **Invariants**: What must be true for safety
3. **Scope**: Minimize the unsafe region
4. **Testing**: Extra tests covering edge cases
