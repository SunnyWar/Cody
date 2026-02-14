# Cody AI Orchestration - Quick Reference

## File Locations

```
Prompts:     .github/ai/prompts/*.md
Scripts:     cody-agent/*.py
Config:      cody-agent/config.json
Launcher:    run_orchestrator.ps1
TODO Lists:  TODO_*.md (generated)
Logs:        orchestrator.log (generated)
```

## Quick Commands

### Run Everything (Automatic)
```powershell
.\run_orchestrator.ps1    # Interactive menu
# OR
cd cody-agent
python orchestrator.py    # Direct execution
```

### Analysis Only (Generate TODO Lists)
```powershell
cd cody-agent
python refactoring_analyzer.py    # → TODO_REFACTORING.md
python performance_analyzer.py    # → TODO_PERFORMANCE.md
python features_analyzer.py       # → TODO_FEATURES.md
```

### Execute Tasks

**Execute specific item:**
```powershell
cd cody-agent
python refactoring_executor.py REF-001
python performance_executor.py PERF-003
python features_executor.py FEAT-005
```

**Execute next available:**
```powershell
cd cody-agent
python refactoring_executor.py next
python performance_executor.py next
python features_executor.py next
```

### View Status
```powershell
cd cody-agent
python todo_manager.py refactoring    # Show stats
python todo_manager.py performance
python todo_manager.py features
```

### Check Results
```powershell
# View TODO lists
cat TODO_REFACTORING.md
cat TODO_PERFORMANCE.md
cat TODO_FEATURES.md

# View execution log
cat orchestrator.log

# Check git history
git log --oneline
```

## Workflow Phases

**1. Refactoring** → Code quality, separation of concerns, architecture
**2. Performance** → Speed optimizations, memory efficiency, algorithmic improvements
**3. Features** → Missing capabilities for world-class engine (max 3 at once)

## Configuration

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
  "github_repo": "yourusername/cody-engine"
}
```

## Environment Setup

```powershell
# Install Agents SDK
pip install openai-agents

# OpenAI API key
$env:OPENAI_API_KEY = "sk-..."

# GitHub token (optional)
$env:GITHUB_TOKEN = "ghp_..."
```

## Quality Gates

Every change must pass:
- ✅ `cargo fmt` - Format
- ✅ `cargo build --release` - Build
- ✅ `cargo test` - Tests
- ✅ `cargo run --release -p engine -- perft 5` - Move gen validation

## TODO Item Structure

```json
{
  "id": "REF-001",              // Unique ID (REF/PERF/FEAT-###)
  "title": "Brief description",
  "priority": "high",           // critical|high|medium|low
  "category": "...",            // Category-specific
  "status": "not-started",      // not-started|in-progress|completed
  "estimated_complexity": "medium",  // small|medium|large
  "files_affected": [...],
  "dependencies": [...],
  "description": "...",
  ...
}
```

## Status Flow

```
not-started → in-progress → completed
```

## Git Checkpoints

After each successful change:
```
Checkpoint: Refactoring: REF-001
Checkpoint: Performance: PERF-003
Checkpoint: Feature: FEAT-005
```

## Troubleshooting

**Invalid patches:**
- Check model temperature in config
- Verify sufficient context in prompts

**Tests fail:**
- Auto-rollback occurs
- Check temp_*.patch for manual review

**Duplicates:**
- Improve duplicate detection
- Manually prune TODO lists

**Long runtime:**
- Reduce max features (default: 3)
- Run individual phases
- Skip phases by commenting out

## Architecture Constraints

**MUST RESPECT:**
- Fixed-block arena (preallocated nodes)
- Allocation-free hot paths
- bitboard/engine crate separation
- Type safety with newtypes

## Logs & Output

```
orchestrator.log      # Full execution log
TODO_*.md            # Human-readable TODO lists
.todo_*.json         # Machine-readable TODO lists
temp_*.patch         # Failed patches (for review)
```

## Common Workflows

**Daily improvement run:**
```powershell
.\run_orchestrator.ps1
# Select 1
```

**Selective execution:**
```powershell
.\run_orchestrator.ps1
# Select 2 (analyze)
# Review TODO_*.md
# Select 3 (execute next from each)
```

**Status check:**
```powershell
.\run_orchestrator.ps1
# Select 4
```

## File Purposes

| File | Purpose |
|------|---------|
| `orchestrator.py` | Master coordinator |
| `*_analyzer.py` | Generate TODO lists |
| `*_executor.py` | Execute specific tasks |
| `todo_manager.py` | TODO list utilities |
| `config.json` | AI configuration |
| `run_orchestrator.ps1` | Interactive launcher |

## Dependencies

**Agent system:**
- Must resolve before executing task
- Specified in `dependencies` field
- Format: `["REF-002", "PERF-001"]`

**Execution order:**
- Priority: critical > high > medium > low
- Dependencies: only execute when met
- Status: not-started items only

## Success Indicators

**After orchestrator run:**
- ✅ All TODO lists populated
- ✅ Changes committed to git
- ✅ All validations passed
- ✅ orchestrator.log shows no errors
- ✅ Repo in clean buildable state

## Need Help?

1. Check `cody-agent/README.md` for full docs
2. Review prompt files in `.github/ai/prompts/`
3. Check `orchestrator.log` for errors
4. Review git history: `git log`

## Key URLs

- Chess Programming Wiki: https://www.chessprogramming.org/
- UCI Protocol: https://www.wbec-ridderkerk.nl/html/UCIProtocol.html
- Stockfish: https://github.com/official-stockfish/Stockfish
- Leela: https://github.com/LeelaChessZero/lc0
