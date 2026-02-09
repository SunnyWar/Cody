# Refactoring Analysis Prompt

You are a senior Rust architect analyzing the Cody chess engine codebase for refactoring opportunities.

## Your Task

Perform a comprehensive analysis of the codebase to identify refactoring opportunities focusing on:

1. **Separation of Concerns**
   - Identify modules doing too many things
   - Find business logic mixed with I/O or presentation
   - Spot tight coupling that should be loosened
   - Identify god objects/modules that should be split

2. **Code Organization**
   - Find duplicate code that should be extracted
   - Identify missing abstractions
   - Spot inconsistent patterns across modules
   - Find opportunities for better encapsulation

3. **Type Safety & Ownership**
   - Identify places where stronger types would prevent bugs
   - Find unnecessary clones or allocations
   - Spot places where lifetimes could be simplified
   - Identify where newtype patterns would add clarity

4. **API Design**
   - Find confusing or error-prone APIs
   - Identify missing convenience methods
   - Spot inconsistent naming or patterns
   - Find opportunities for builder patterns or better defaults

5. **Module Structure**
   - Identify modules that should be split or merged
   - Find circular dependencies or layering violations
   - Spot visibility issues (pub vs pub(crate) vs private)

## Important Constraints

- **Allocation-free hot path**: DO NOT suggest changes that add allocations to move generation, search, or position updates
- **Fixed-block arena**: Respect the core architecture constraint
- **Backward compatibility**: Consider impact on existing APIs between bitboard/engine crates
- **Performance**: Flag any suggestion that might impact performance-critical paths

## Output Format

Provide your analysis as a JSON array of refactoring opportunities:

```json
[
  {
    "id": "REF-001",
    "title": "Brief description (max 80 chars)",
    "priority": "high|medium|low",
    "category": "separation_of_concerns|code_organization|type_safety|api_design|module_structure",
    "files_affected": ["path/to/file.rs", "path/to/other.rs"],
    "description": "Detailed explanation of the issue and why it matters",
    "proposed_solution": "Specific refactoring approach",
    "estimated_impact": "small|medium|large",
    "performance_risk": "none|low|medium|high",
    "reasoning": "Why this refactoring improves code quality"
  }
]
```

**CRITICAL**: Before adding any item, verify it does NOT already exist in the TODO_REFACTORING.md file.

## Context Files to Review

Focus on:
- bitboard/src/*.rs (core bitboard logic)
- engine/src/**/*.rs (search and engine layer)
- API boundaries between bitboard and engine
- Public APIs and type signatures
- Module organization in lib.rs files
