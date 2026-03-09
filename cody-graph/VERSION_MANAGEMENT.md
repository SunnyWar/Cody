# Version Management Policy

## Overview

The Cody engine uses **semantic versioning** (X.Y.Z) with automatic version management:
- **X (Major)**: Manual only - never auto-incremented
- **Y (Minor)**: Auto-incremented by **elogain** phase (successful ELO improvements)
- **Z (Patch)**: Auto-incremented by **all other phases** (clippy, refactoring, performance, ucifeatures)

## Versioning Rules

### When Minor (Y) Increments:
- **Phase**: elogain
- **Trigger**: Successful ELO improvement committed
- **Example**: 0.1.5 → 0.2.0 (minor++, patch reset to 0)
- **Meaning**: Engine gameplay improved measurably

### When Patch (Z) Increments:
- **Phases**: clippy, refactoring, performance, ucifeatures
- **Trigger**: Any Rust (.rs) file modified
- **Example**: 0.1.5 → 0.1.6 (patch++)
- **Meaning**: Code change that could affect behavior (but not proven ELO gain)

### When Major (X) Increments:
- **Manual only** - requires human decision
- **Examples**: Breaking API changes, major architecture rewrites
- **Not handled by automation**

## Why This Matters

- **Only Rust code affects engine behavior**: The chess engine is written in Rust
- **Minor vs Patch distinction**: 
  - Minor (Y) = proven ELO improvement via gauntlet testing
  - Patch (Z) = code change that could affect behavior
- Changes to other files (Python orchestration, docs, configs) don't affect gameplay
- Rust changes that DO affect behavior include:
  - Move generation logic
  - Search algorithms  
  - Evaluation functions
  - Board representation
  - Performance optimizations (timing, cache behavior)
  - Even refactoring and clippy fixes (can affect codegen)
- Version tracking is essential for:
  - ELO gain testing (comparing engine versions)
  - Regression tracking
  - Tournament play (versioned binaries)
  - Understanding which improvements actually gained ELO

## What Triggers Version Bumps

✅ **DO increment MINOR (Y)** when committing from **elogain** phase:
- Successful gauntlet test results
- Proven ELO improvement
- Example: 0.1.5 → 0.2.0

✅ **DO increment PATCH (Z)** when committing from **other phases** with .rs changes:
- **clippy** phase: ANY `.rs` file changes
- **refactoring** phase: ANY `.rs` file changes
- **performance** phase: ANY `.rs` file changes
- **ucifeatures** phase: ANY `.rs` file changes
- Changes in `bitboard/src/` directory
- Changes in `engine/src/` directory
- Example: 0.1.5 → 0.1.6

❌ **DO NOT increment version** when committing:
- Python script changes (`cody-graph/`, `tools/`)
- Documentation changes (`.md` files)
- Configuration changes (`.json`, `.toml` except version field)
- Test data changes (`.epd`, `.pgn`)
- Build script changes (PowerShell scripts)
- No .rs files modified

## Implementation

### Use `commit_util.py` for ALL Commits

**Location:** `cody-graph/tools/commit_util.py`

**Usage:**
```python
from commit_util import commit_with_version_bump

success, new_version, error = commit_with_version_bump(
    repo_path=".",
    commit_message="Fix clippy warnings",
    phase="clippy",  # or "refactoring", "performance", "elogain", etc.
    files_to_add=None  # None = all modified files, or specify list
)

if not success:
    print(f"Commit failed: {error}")
```

### What It Does

1. **Checks for Rust changes** - Scans staged/modified files for `.rs` extensions
2. **Conditionally increments version** in `engine/Cargo.toml` **only if .rs files changed**:
   - **elogain phase**: Minor version increments (Y++, Z reset to 0)
     - Example: 0.1.5 → 0.2.0
   - **Other phases**: Patch version increments (Z++)
     - Example: 0.1.5 → 0.1.6
3. **Stages files** for commit (specified files or all modified)
4. **Includes `Cargo.toml`** only if version was bumped
5. **Creates commit** with format:
   - If .rs changed: `v{version} - {phase}: {message}` 
     - ELO example: `v0.2.0 - elogain: Improved move ordering`
     - Other example: `v0.1.6 - clippy: Remove unused imports`
   - If no .rs changed: `{phase}: {message}` 
     - Example: `clippy: Update documentation`

### Phases That Must Use This

✅ **Clippy Phase** - When committing warning fixes  
✅ **Refactoring Phase** - When committing code improvements  
✅ **Performance Phase** - When committing optimizations  
✅ **UCI Features Phase** - When committing protocol enhancements  
✅ **ELO Gain Phase** - When committing gameplay improvements (already implemented)

## Current Status

- ✅ **ELO Gain Agent** - Updated to use `commit_util.py`
- ✅ **commit_or_revert.py** - Updated to use `commit_util.py`

Remaining migration tasks are tracked in the root `TODO.md` document.

## Binary Management

After each commit with version bump:

1. **Build release binary:**
   ```bash
   cargo build --release
   ```

2. **Copy to engines directory:** (optional, for ELO testing)
   ```python
   from version_manager import copy_binary_with_version
   
   copy_binary_with_version(
       "target/release/cody.exe",
       "C:\\chess\\Engines",
       new_version
   )
   ```

## Version Scheme

**Format:** `MAJOR.MINOR.PATCH`

- **MAJOR** (0.x.x) - Reserved for major architecture changes
- **MINOR** (x.1.x) - Reserved for feature additions
- **PATCH** (x.x.1) - **AUTO-INCREMENTED** for every commit

### Rationale

- Patch version automatically increments to ensure every commit has a unique version
- This enables precise tracking of which code produced which behavior
- Critical for ELO gauntlet testing (candidate vs champion versions)

## Git History Example

```
v0.1.8 - ELOGain: ELO gain (+5.2 ELO estimated)
v0.1.7 - performance: Optimize transposition table lookup
v0.1.6 - refactoring: Extract move ordering into separate function
v0.1.5 - clippy: Remove unused imports in evaluator
v0.1.4 - clippy: Fix deprecated syntax in UCI parser
```

## For Agent Developers

When implementing new agents or phases:

1. **Import commit_util:**
   ```python
   import sys
   from pathlib import Path
   sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
   from commit_util import commit_with_version_bump
   ```

2. **Make changes to code** (apply diffs, refactor, etc.)

3. **Commit with version bump:**
   ```python
   success, new_version, error = commit_with_version_bump(
       repo_path=state.get("repo_path", "."),
       commit_message="Your descriptive message here",
       phase=state.get("current_phase", "general"),
       files_to_add=None  # or list of specific files
   )
   ```

4. **Handle result:**
   ```python
   if success:
       state["committed_version"] = new_version
       state["status"] = "ok"
   else:
       state["status"] = "error"
       state["error_message"] = error
   ```

## Exception: No-Commit Phases

Some phases may not commit changes directly (e.g., running benchmarks, analysis only). These phases don't need version management.

## See Also

- [version_manager.py](elo_tools/version_manager.py) - Version reading/writing utilities
- [commit_util.py](tools/commit_util.py) - Centralized commit utility
- [elo_gain_agent.py](agents/elo_gain_agent.py) - Example usage in ELO Gain phase
