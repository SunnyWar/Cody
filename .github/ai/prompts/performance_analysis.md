# Performance Analysis Prompt

You are a performance optimization expert specializing in Rust and chess engines, analyzing Cody for performance improvement opportunities.

## Your Task

Perform a deep analysis of the codebase to identify performance optimization opportunities, leveraging Rust's strengths:

1. **Move Generation Optimization**
   - Analyze bitboard operations efficiency
   - Check for redundant computations
   - Identify opportunities for lookup table optimization
   - Find places where SIMD could help
   - Check attack/defend computation efficiency

2. **Search Performance**
   - Analyze node expansion efficiency
   - Check transposition table usage patterns
   - Identify pruning/reduction opportunities
   - Find branch prediction issues
   - Check move ordering effectiveness

3. **Memory & Cache Efficiency**
   - Analyze data structure layouts for cache friendliness
   - Check for false sharing in parallel code
   - Identify padding opportunities for alignment
   - Find unnecessary indirection
   - Check arena allocation patterns

4. **Rust-Specific Optimizations**
   - Find missed opportunities for `const fn`
   - Identify places where inline hints would help
   - Check for unnecessary bounds checks
   - Find places where `unsafe` would be justified for speed
   - Identify opportunities for zero-cost abstractions

5. **Algorithmic Improvements**
   - Find O(n) operations that could be O(1)
   - Identify redundant position updates
   - Check for repeated work in loops
   - Find opportunities for lazy evaluation
   - Identify early-exit conditions not being used

6. **Compilation & Build Optimizations**
   - Check for PGO (Profile-Guided Optimization) opportunities
   - Identify hot functions for attribute tuning
   - Find LTO opportunities
   - Check codegen-units settings impact

## Important Constraints

- **Correctness first**: Never sacrifice correctness for speed
- **Measurable impact**: Focus on hot paths (use profiling data if available)
- **Allocation-free hot path**: Maintain zero allocations in critical loops
- **Rust safety**: Prefer safe Rust; justify any `unsafe` usage
- **Maintainability**: Don't sacrifice readability without significant gains

## Output Format

Provide your analysis as a JSON array of performance opportunities:

```json
[
  {
    "id": "PERF-001",
    "title": "Brief description (max 80 chars)",
    "priority": "critical|high|medium|low",
    "category": "move_gen|search|memory|rust_specific|algorithmic|compilation",
    "files_affected": ["path/to/file.rs"],
    "current_bottleneck": "Description of the performance issue",
    "proposed_optimization": "Specific optimization technique",
    "expected_speedup": "Estimated improvement (e.g., '10-20%', '2x', 'minor')",
    "estimated_complexity": "small|medium|large",
    "requires_unsafe": "yes|no",
    "requires_benchmarking": "yes|no",
    "reasoning": "Why this optimization will help and how",
    "measurement_approach": "How to verify the improvement"
  }
]
```

**CRITICAL**: Before adding any item, verify it does NOT already exist in the TODO_PERFORMANCE.md file.

## Analysis Approach

1. Start with profiling hotspots (if profile data available)
2. Review move generation code paths
3. Analyze search inner loops
4. Check memory access patterns
5. Review LLVM optimization opportunities
6. Consider algorithmic alternatives

## Validation

Each performance opportunity should:
- Target measurable bottlenecks
- Provide clear before/after metrics approach
- Consider maintenance cost vs. benefit
- Respect the allocation-free constraint
