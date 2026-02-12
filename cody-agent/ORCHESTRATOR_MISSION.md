# Orchestrator Mission & Workflow

## Core Mission
The orchestrator runs automated code improvement cycles. Each run:
1. Finds ONE improvement opportunity (refactoring, performance, feature, or clippy warning)
2. Calls an LLM to generate the fix/implementation
3. Applies the code changes
4. Validates with tests/builds
5. Commits to git
6. Exits (next run picks up the next task)

## Phase Flow
1. **Refactoring** → clippy cleanup → **Performance** → clippy cleanup → **Features** (max 3) → clippy cleanup → Done
2. Clippy runs between each main phase to clean up any warnings introduced

## Standard Executor Pattern
All executors (refactoring, performance, features, clippy) follow this pattern:

### 1. Analyzer Phase
- Runs analysis tool (clippy, benchmarks, code review)
- Generates TODO items with structured metadata
- Saves to `.todo_<category>.json`
- Returns: count of items added

### 2. Executor Phase
- Loads next TODO item from list
- Gathers file content for context
- Builds prompt with:
  - Task description from TODO
  - Affected file(s) full content
  - Specific instructions (return full file in ```rust block with file path comment)
- Calls LLM
- Extracts updated file content from response
- Writes file(s) to disk
- Validates (cargo build, cargo test)
- Marks TODO item complete
- Returns: success/failure

### 3. Orchestrator Integration
```python
# Analysis (once per phase)
if not self.state["analysis_done"]:
    added = analyzer.analyze(repo_root, config)
    self.state["analysis_done"] = True
    
# Execute one task
todo_list = TodoList(category, repo_root)
next_item = todo_list.get_next_item()

if not next_item:
    # Phase complete, advance to next
    self._advance_main_phase(current_phase)
    return self.run_single_improvement()

# Execute the task
success = executor.execute_task(next_item.id, repo_root, config)

if success and self._has_code_changes():
    self._create_checkpoint(f"{category}: {next_item.id}")
    return True  # Exit, next run continues
```

## Critical Rules
1. **One task per run** - orchestrator exits after each successful code change
2. **LLM always gets full file content** - not snippets, not patches
3. **LLM returns full updated file** - in ```rust block with file path comment
4. **Validate before commit** - cargo build && cargo test must pass
5. **No code changes = continue to next task** - don't exit if only TODO files changed

## Clippy-Specific Flow
Clippy warnings are treated like any other task category:

1. **Analyzer**: Run `clippy_parser.py` → parse JSON warnings → create TODO items
2. **Executor**: 
   - Load TODO item
   - Read affected file
   - Send to LLM with warning details + full file content
   - Extract updated file from LLM response
   - Apply changes
   - Validate with cargo
   - Mark complete
3. **Orchestrator**: Same pattern as other phases

## File Structure
- `orchestrator.py` - Main loop, phase management
- `{category}_analyzer.py` - Find opportunities, create TODOs
- `{category}_executor.py` - Execute one task with LLM
- `todo_manager.py` - Shared TODO list management
- `.todo_{category}.json` - Persistent task tracking
- `orchestrator_state.json` - Phase tracking, run count

## Common Mistakes to Avoid
1. ❌ Calling LLM in analyzer (analysis should be deterministic)
2. ❌ Returning JSON from LLM for code (always return full files in code blocks)
3. ❌ Executing multiple tasks per run (one and done)
4. ❌ Skipping validation (always run cargo build + test)
5. ❌ Forgetting file path comment in LLM prompt (needed for extraction)
