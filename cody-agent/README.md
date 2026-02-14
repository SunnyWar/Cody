# Cody AI Agent Orchestration System

A sophisticated multi-agent system for **automated, iterative improvement** of the Cody chess engine through systematic refactoring, performance optimization, and feature implementation.

## Table of Contents

- [Quick Start](#quick-start)
- [How It Works](#how-it-works)
- [Installation](#installation)
- [Usage](#usage)
- [Architecture](#architecture)
- [Workflow Details](#workflow-details)
- [Configuration](#configuration)
- [Quality Gates](#quality-gates)
- [Troubleshooting](#troubleshooting)

## Quick Start

### One-Command Single Improvement (Run Repeatedly)

```powershell
# Windows/PowerShell (recommended)
.\run_orchestrator.ps1

# Linux/macOS
cd cody-agent
python orchestrator.py
```

**Key point:** Each run produces **exactly one code improvement** and exits. 
Call this script repeatedly (via scheduled task, cron, etc.) to make continuous progress.

- First run: One refactoring committed
- Second run: Next refactoring committed
- ...iterate until all refactorings done, then performance, then features

All changes are validated, tested, and committed automatically.

### View Progress

```powershell
cd cody-agent
python todo_manager.py refactoring    # Show refactoring progress
python todo_manager.py performance    # Show optimization progress
python todo_manager.py features       # Show feature progress
python todo_manager.py clippy         # Show clippy warnings progress
```

### Check Results

```powershell
cat TODO_REFACTORING.md    # Review refactoring opportunities
cat TODO_PERFORMANCE.md    # Review optimization opportunities
cat TODO_FEATURES.md       # Review feature opportunities
cat TODO_CLIPPY.md         # Review clippy warnings
cat orchestrator.log       # Complete execution log
git log --oneline          # View git checkpoints
```

## How It Works

The orchestration system implements a **multi-phase improvement workflow** with Clippy integration:

### Phase 1: Refactoring ♻️

1. **Analyze** the codebase for refactoring opportunities
   - Separation of concerns issues
   - Code organization improvements
   - Type safety opportunities
   - API design improvements
   - Module structure optimization

2. **Execute** each refactoring task
   - Generates patch via AI
   - Applies with `git apply`
   - Validates with quality gates
   - Commits on success

3. **Repeat** until all refactorings complete

4. **Clippy Check** - Fix any clippy warnings introduced

### Phase 2: Performance Optimization ⚡

1. **Analyze** for performance improvements
   - Move generation bottlenecks
   - Search hot paths
   - Memory & cache efficiency
   - Rust-specific optimizations
   - Algorithmic improvements

2. **Execute** each optimization
   - Generates patch via AI
   - Benchmarks before & after
   - Validates correctness
   - Commits on success

3. **Repeat** until all optimizations complete

4. **Clippy Check** - Fix any clippy warnings introduced

### Phase 3: World-Class Features ✨

1. **Analyze** what features are needed
   - Compare to Stockfish, Leela, etc.
   - Identify missing search enhancements
   - Identify missing evaluation improvements
   - Identify missing UCI features

2. **Execute** up to 3 features
   - For each feature:
     - Implement the feature via AI
     - If diff > 100 lines: Re-run Phases 1 & 2
     - If diff ≤ 100 lines: Continue to next feature

3. **Stop** after 3 features or no more left

4. **Clippy Check** - Fix any clippy warnings introduced

### Clippy Integration 🔧

After each main phase, the system runs `cargo clippy` and:
- Analyzes warnings with focus on performance and memory
- Creates TODO items for actionable fixes
- Executes fixes one at a time
- Validates each fix passes all quality gates

## Installation

### Prerequisites

- Python 3.8+
- Rust toolchain (for building/testing)
- Git
- PowerShell (Windows) or Bash (Linux/macOS)

### Setup Steps

1. **Install Agents SDK:**
   ```powershell
   pip install openai-agents
   ```

2. **Configure AI model** (edit `cody-agent/config.json`):
   ```json
   {
     "model": "gpt-5.3-codex",
     "use_local": false,
     "skills": {
       "enabled": ["github_fix_ci", "github_address_comments"],
       "run_timing": "after",
       "ci_log_path": ".orchestrator_logs/ci_failure.txt",
       "pr_comments_path": ".orchestrator_logs/pr_review_comments.json"
     }
   }
   ```

3. **Set environment variables:**
   ```powershell
   $env:OPENAI_API_KEY = "sk-..."
   $env:GITHUB_TOKEN = "ghp_..."
   ```

4. **Verify setup:**
   ```powershell
   cd cody-agent
   python orchestrator.py --help
   ```

## Usage

### Automatic Full Workflow

**Interactive menu (easiest):**
```powershell
.\run_orchestrator.ps1
# Select option 1 for full workflow
```

**Direct execution:**
```powershell
cd cody-agent
python orchestrator.py
```

#### Important: Run the Script Repeatedly

The orchestrator is **designed to run repeatedly** (e.g., via scheduled task). Here's how it works:

- **Each invocation produces exactly ONE code change**
- The script analyzes, executes one task, commits, and **exits**
- State is preserved in `orchestrator_state.json` between runs
- The next invocation continues from where the previous one left off

**Example: Scheduled daily runs:**
```powershell
# Schedule this PowerShell script to run N times per day (e.g., every 8 hours)
.\run_orchestrator.ps1

# Result after each run:
# ✅ One refactoring/optimization/feature merged to git
# ✅ Log updated with progress
# ✅ State saved for next run
```

**Progress through phases:**
1. First run: Executes one refactoring (if any exist)
2. Second run: Executes next refactoring (or moves to performance phase)
3. ...continues until all refactorings done
4. Then proceeds to performance optimizations (one per run)
5. Then proceeds to features (up to 3 total, one per run)

**Note:** Only actual code changes (.rs files, config files) count. Tasks that only update TODO lists are automatically skipped and the orchestrator continues to the next task.

The orchestrator will:
- Track which phase you're in (refactoring → performance → features)
- Execute one improvement per invocation
- Create git checkpoints after each code change
- Log all actions to `orchestrator.log`
- Exit cleanly after each merge

### Manual Workflow (Fine-Grained Control)

**Analyze only (generate TODO lists without executing):**
```powershell
cd cody-agent
python refactoring_analyzer.py    # → TODO_REFACTORING.md
python performance_analyzer.py    # → TODO_PERFORMANCE.md
python features_analyzer.py       # → TODO_FEATURES.md
python clippy_analyzer.py         # → TODO_CLIPPY.md
```

**Execute specific item:**
```powershell
cd cody-agent
python refactoring_executor.py REF-001     # Execute specific refactoring
python performance_executor.py PERF-003    # Execute specific optimization
python features_executor.py FEAT-005       # Execute specific feature
python clippy_executor.py CLIPPY-001       # Execute specific clippy fix

# Finalize the commit for the executor change
python commit_executor_change.py
```

**Execute next available item:**
```powershell
cd cody-agent
python refactoring_executor.py next        # Next refactoring
python performance_executor.py next        # Next optimization
python features_executor.py next           # Next feature
python clippy_executor.py next             # Next clippy fix

# Finalize the commit for the executor change
python commit_executor_change.py
```

**View TODO statistics:**
```powershell
cd cody-agent
python todo_manager.py refactoring
python todo_manager.py performance
python todo_manager.py features
python todo_manager.py clippy
```

## Architecture

### System Components

| Component | Purpose |
|-----------|---------|
| **orchestrator.py** | Master coordinator - runs all phases |
| **refactoring_analyzer.py** | Analyzes code for refactoring opportunities |
| **refactoring_executor.py** | Implements specific refactorings |
| **performance_analyzer.py** | Analyzes code for performance improvements |
| **performance_executor.py** | Implements specific optimizations |
| **features_analyzer.py** | Analyzes missing world-class features |
| **features_executor.py** | Implements specific features |
| **clippy_analyzer.py** | Analyzes clippy warnings for actionable fixes |
| **clippy_executor.py** | Fixes specific clippy warnings |
| **todo_manager.py** | Manages TODO lists (load, save, validate, track) |

### Prompt Templates

Located in `.github/ai/prompts/` (beside `system.md`):

| File | Purpose |
|------|---------|
| **orchestrator.md** | Master coordination guidelines |
| **refactoring_analysis.md** | How to identify refactoring opportunities |
| **refactoring_execution.md** | How to implement refactorings |
| **performance_analysis.md** | How to identify performance improvements |
| **performance_execution.md** | How to implement optimizations |
| **features_analysis.md** | How to identify missing features |
| **features_execution.md** | How to implement features || **clippy_analysis.md** | How to analyze clippy warnings |
| **clippy_execution.md** | How to fix clippy warnings |
## Workflow Details

### Phase 1: Refactoring (Complete Until Done)

**Step 1a: Analysis**
- Reads all Rust source files
- Analyzes architecture and design patterns
- AI identifies refactoring opportunities
- Generates `TODO_REFACTORING.md` with prioritized list

**Step 1b: Execution Loop**
- Gets next highest-priority refactoring
- AI implements the change as a unified diff patch
- Patch is applied with `git apply`
- Quality gates run automatically
- If all pass: git checkpoint created, item marked completed
- If any fail: changes rolled back, item skipped
- Repeat until all refactorings complete

**Quality Checks:**
- ✅ `cargo fmt` - Code formatting
- ✅ `cargo build --release` - Release build
- ✅ `cargo test` - All unit tests
- ✅ `cargo run --release -p engine -- perft 5` - Move generation validation

### Phase 2: Performance Optimization (Complete Until Done)

**Step 2a: Analysis**
- Analyzes move generation hot paths
- Analyzes search hot loops
- Checks memory and cache efficiency
- Identifies Rust-specific optimizations
- Generates `TODO_PERFORMANCE.md` with estimated speedup

**Step 2b: Execution Loop**
- Gets next highest-priority optimization
- AI implements the optimization
- Benchmarks are run before and after
- Validates correctness (all tests must pass)
- If speedup confirmed: git checkpoint, item marked completed
- If no improvement: changes rolled back, item skipped
- Repeat until all optimizations complete

**Validation:**
- Same quality checks as Phase 1
- Plus: before/after benchmark comparison
- Minimum 5% speedup required (or architectural justification)

### Phase 3: World-Class Features (Limited to 3)

**Step 3a: Analysis**
- Analyzes current implementation
- Compares to world-class engines
- Identifies missing features (search, evaluation, UCI, etc.)
- Generates `TODO_FEATURES.md` with ELO impact estimates

**Step 3b: Execution (Max 3 Features)**
- For each of up to 3 features:
  - AI implements the feature
  - Quality checks run automatically
  - If diff is LARGE (>100 lines):
    - Feature is committed
    - Phases 1 & 2 re-run entirely
    - Then continue to next feature
  - If diff is SMALL (≤100 lines):
    - Feature is committed
    - Continue immediately to next feature

This prevents feature changes from introducing architecture debt.

## TODO List Management

### Generated Files

The system maintains three separate TODO lists, each in dual format:

```
TODO_REFACTORING.md        Human-readable refactoring TODO list
.todo_refactoring.json     Machine-readable version (for programmatic access)

TODO_PERFORMANCE.md        Human-readable optimization TODO list
.todo_performance.json     Machine-readable version

TODO_FEATURES.md           Human-readable features TODO list
.todo_features.json        Machine-readable version

TODO_CLIPPY.md             Human-readable clippy warnings TODO list
.todo_clippy.json          Machine-readable version
```

### TODO Item Structure

Each item tracks:

```json
{
  "id": "REF-001",
  "title": "Extract move ordering logic",
  "priority": "high",
  "category": "separation_of_concerns",
  "description": "Detailed explanation of the refactoring...",
  "status": "not-started",
  "estimated_complexity": "medium",
  "files_affected": ["engine/src/search/search.rs"],
  "dependencies": [],
  "created_at": "2026-02-08T10:00:00",
  "completed_at": null
}
```

### Status Lifecycle

```
not-started → in-progress → completed
```

- **not-started**: Detected but not yet undertaken
- **in-progress**: Currently being implemented
- **completed**: Successfully implemented and validated

### Smart Features

**Duplicate Detection:**
- Prevents re-adding the same task
- Checks title similarity and file overlap
- Validates against existing TODO lists

**Dependency Tracking:**
- Tasks can depend on other tasks
- Skip items with unmet dependencies
- Execute in proper order

**Priority Ordering:**
- `critical` > `high` > `medium` > `low`
- Execute highest priority items first
- Respects dependencies

## Configuration

### config.json

Edit `cody-agent/config.json`:

```json
{
   "model": "gpt-5.1",
   "models": {
      "refactoring": "gpt-5.1",
      "features": "gpt-5.1",
      "logic_bugs": "o3",
      "clippy": "gpt-5-mini",
      "unit_tests_docs": "gpt-5-nano"
   },
   "use_local": false,
   "skills": {
      "enabled": ["github_fix_ci", "github_address_comments"],
      "run_timing": "after",
      "ci_log_path": ".orchestrator_logs/ci_failure.txt",
      "pr_comments_path": ".orchestrator_logs/pr_review_comments.json"
   },
  "branch_prefix": "ai-feature-",
  "github_repo": "yourusername/cody-engine"
}
```

**Configuration Options:**

| Option | Type | Description |
|--------|------|-------------|
| `use_local` | bool | Reserved for future local provider support |
| `model` | string | OpenAI model name for the Agents SDK |
| `models` | object | Task-specific model overrides |
| `skills.enabled` | list | Enabled skills (`github_fix_ci`, `github_address_comments`) |
| `skills.run_timing` | string | When to run skills (`after`) |
| `skills.ci_log_path` | string | CI failure log file path |
| `skills.pr_comments_path` | string | PR review comments path |
| `branch_prefix` | string | Git branch name prefix |
| `github_repo` | string | GitHub repo (for future PR integration) |

### Environment Variables

**For OpenAI authentication:**
```powershell
$env:OPENAI_API_KEY = "sk-..."
```

**For GitHub integration:**
```powershell
$env:GITHUB_TOKEN = "ghp_..."
```

### AI Model Examples

**OpenAI Models:**
- `gpt-5.3-codex`
- `o4-mini`

## Quality Gates

Every change is automatically validated before being committed. All of these must pass:

### 1. Code Formatting
```powershell
cargo fmt --check
```
Ensures consistent code style across the project.

### 2. Debug Build
```powershell
cargo build
```
Verifies the code compiles in debug mode.

### 3. Release Build
```powershell
cargo build --release
```
Verifies optimized compilation with LTO enabled.

### 4. Unit Tests
```powershell
cargo test
```
All existing tests must pass without regression.

### 5. Move Generation Validation (Perft)
```powershell
cargo run --release -p engine -- perft 5
```
Validates that move generation hasn't been altered.

**Only if all checks pass:** Changes are committed to git with a checkpoint.  
**If any check fails:** Changes are automatically rolled back.

## Architecture Constraints

The system respects Cody's core architectural principles:

| Constraint | Reason | Impact on Agents |
|-----------|--------|------------------|
| **Fixed-block arena** | Search nodes preallocated | Never suggest dynamic allocation in search |
| **Allocation-free hot path** | No heap allocs in loops | Skip refactorings that add allocations to movegen/search |
| **Crate separation** | Clear responsibilities | bitboard = board logic, engine = search/UCI |
| **Type safety** | Prevent bugs | Prefer strong newtypes over primitives |

Agents are explicitly instructed to respect these constraints.

## How Changes Are Applied

### The Change Pipeline

```
1. AI Generates Code
   ↓
2. Parse as Unified Diff Patch
   ↓
3. Apply with git apply
   ↓
4. Run Quality Gates (5 checks)
   ↓
5a. ALL PASS             5b. ANY FAIL
    ↓                        ↓
    Create Git Checkpoint    Rollback Changes
    Mark Task Complete       Skip Task
    Continue                 Try Next Task
```

### Git Checkpoints

Each successful change creates a checkpoint:

```
commit abc1234
Author: Cody AI <ai@cody.local>
Date:   2026-02-08 14:30:00

    Checkpoint: Refactoring: REF-001 - Extract move ordering logic
```

This allows:
- Easy rollback to any previous state
- Clear audit trail of improvements
- Individual review of changes
- Bisecting to find regressions

## Troubleshooting

### AI generates invalid patches

**Symptom:** `git apply` fails or patch doesn't compile

**Cause:** Model context incomplete or prompt unclear

**Solution:**
1. Check the `temp_*.patch` file for details
2. Reduce model temperature for determinism
3. Verify sufficient code context in prompt
4. Item stays in TODO for manual review

### Tests fail after change

**Symptom:** One or more quality gates fail

**Cause:** Generated code has bugs or regressions

**Solution:**
- Changes are automatically rolled back
- Check `orchestrator.log` for error details
- Item remains in TODO for manual implementation
- Try rerunning (AI might generate different code)

### Duplicate items in TODO lists

**Symptom:** Similar items appear in TODO lists

**Cause:** Duplicate detection needs tuning

**Solution:**
1. Improve duplicate detection logic in `todo_manager.py`
2. Make analyzer prompts more explicit about checking
3. Manually prune duplicate TODO lists
4. Re-run analyzers

### Very long runtime

**Symptom:** Orchestrator takes hours to complete

**Cause:** Many items or complex patches to apply

**Solution:**
1. Run individual analyzers only (skip execution)
2. Reduce max features: Edit `orchestrator.py` Phase 3 to `max_features=1`
3. Run agents manually for specific items
4. Use faster models (smaller, more efficient)
5. Skip phases by commenting out in `orchestrator.py`

### Patch conflicts

**Symptom:** `git apply` says patch already applied or conflicts

**Cause:** Git state not clean or codebase changed

**Solution:**
- Ensure clean git state: `git status` should show nothing
- Reset if needed: `git reset --hard HEAD`
- Check for incomplete commits: `git log`

### Out of memory during benchmarking

**Symptom:** `cargo bench` runs out of memory

**Cause:** Criterion benchmark harness uses significant memory

**Solution:**
1. Skip performance phase: Comment out `step_2_performance()` in `orchestrator.py`
2. Run benchmarks separately with timeout: `timeout 300 cargo bench -p engine`
3. Close other applications to free memory

## Examples

### Full Orchestration Run

```powershell
$ cd cody-agent
$ python orchestrator.py

# ... processes all three phases ...

📊 Final TODO Statistics:

REFACTORING:
  Total: 5
  Completed: 5

PERFORMANCE:
  Total: 3
  Completed: 3

FEATURES:
  Total: 8
  Completed: 3

CLIPPY:
  Total: 12
  Completed: 12

✅ Orchestrator finished successfully
```

### Manual Workflow

```powershell
# Analyze only
$ python refactoring_analyzer.py
$ python performance_analyzer.py
$ python features_analyzer.py
$ python clippy_analyzer.py

# Review what was found
$ cat TODO_REFACTORING.md
$ cat TODO_PERFORMANCE.md
$ cat TODO_FEATURES.md
$ cat TODO_CLIPPY.md

# Execute just one refactoring
$ python refactoring_executor.py REF-001
✅ Refactoring REF-001 completed successfully

# Execute next available
$ python refactoring_executor.py next
$ python performance_executor.py next
$ python features_executor.py next
$ python clippy_executor.py next
```

## File Structure

```
cody-agent/
├── README.md                       This file
├── config.json                     AI configuration
├── agent.py                        Legacy agent (not used)
├── orchestrator.py                 Master orchestrator
├── todo_manager.py                 TODO list utilities
│
├── refactoring_analyzer.py         Find refactoring opportunities
├── refactoring_executor.py         Execute refactorings
├── performance_analyzer.py         Find performance improvements
├── performance_executor.py         Execute optimizations
├── features_analyzer.py            Find missing features
├── features_executor.py            Implement features
├── clippy_analyzer.py              Analyze clippy warnings
└── clippy_executor.py              Fix clippy warnings

(Generated at runtime)
├── TODO_REFACTORING.md
├── TODO_PERFORMANCE.md
├── TODO_FEATURES.md
├── TODO_CLIPPY.md
├── .todo_refactoring.json
├── .todo_performance.json
├── .todo_features.json
├── .todo_clippy.json
└── orchestrator.log
```

## Tips for Success

### 1. Keep Git Clean
```powershell
git status  # Should show nothing
```

### 2. Monitor Early Runs
Watch the first orchestration run to understand the workflow.

### 3. Review Generated TODOs
Check TODO files before executing.

### 4. Validate Config
Test your AI config before long runs.

### 5. Archive Logs
Keep orchestrator.log after runs for reference.

## License

Same as Cody chess engine (see root LICENSE).

---

**Last Updated:** 2026-02-09  
**Orchestrator Version:** 1.0  
**Compatible with:** Cody chess engine (fixed-block arena architecture)
