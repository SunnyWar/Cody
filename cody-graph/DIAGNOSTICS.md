# Cody-Graph Diagnostics Guide

## Overview
The enhanced cody-graph system now provides detailed console output and persistent logging to help diagnose issues with automated code improvements.

## Console Output Improvements

### Before (Minimal)
```
[cody-graph] run_clippy: start
[cody-graph] run_clippy: end (error)
[cody-graph] clippy_agent: start
[cody-graph] clippy_agent: end (ok)
[cody-graph] apply_diff: start
[cody-graph] apply_diff: error: No valid patches in input
```

### After (Diagnostic)
The system now prints:
- `[DIAG]` tags for diagnostic information
- Model being used
- File/line numbers with warnings
- Context sizes
- Patch tool selection
- Exit codes and error details

Example:
```
[cody-graph] run_clippy: START
[cody-graph] [DIAG] Running 'cargo clippy --'...
[cody-graph] [DIAG] Warnings: 5, Errors: 0
[cody-graph] [DIAG] Exit code: 1
[cody-graph] [DIAG] Saved clippy output to: .../.cody_logs/20260303_120530_clippy_output.txt
[cody-graph] [DIAG] First warning: ...
```

## Log Files

All diagnostic data is saved to `.cody_logs/` directory with timestamps:

### `<timestamp>_clippy_output.txt`
Raw output from `cargo clippy --` command
- Exit codes
- Full warning/error messages
- Compiler output

### `<timestamp>_llm_response.txt`
Complete LLM interaction including:
- System prompt sent to the API
- Input context (warning + code snippet)
- Full response from the model

### `<timestamp>_diff_extracted.log`
The unified diff extracted from LLM response before patch application

### `<timestamp>_patch_stdout.log` / `<timestamp>_patch_stderr.log`
Output from the patch tool (`git apply` or `patch` command)

### `<timestamp>_diff_extraction_failed.log`
Full LLM response when diff extraction fails (for analysis)

### `<timestamp>_patch_exception.log`
Exception stack traces from patch application errors

## Workflow for Troubleshooting

### Scenario 1: "No valid patches in input"
1. Check `.cody_logs/<timestamp>_llm_response.txt`
   - Verify system prompt is clear
   - Check if LLM understood the warning
   - Look for diff block markers (``` ```diff)
   
2. Check `.cody_logs/<timestamp>_diff_extracted.log`
   - Verify the diff format is valid unified diff
   - Check file paths match the repo structure
   
3. Check `.cody_logs/<timestamp>_patch_stderr.log`
   - See exact error from patch tool

### Scenario 2: Patch applied but build/tests failed
1. Check `.cody_logs/<timestamp>_diff_extracted.log` for the applied change
2. Run `cargo build` manually to see exact errors
3. Adjust the system prompt in `clippy_agent.py` if needed

### Scenario 3: LLM response is incomplete or wrong
1. Check `.cody_logs/<timestamp>_llm_response.txt`
2. Look at the input context - is the warning clear enough?
3. Verify model is specified correctly in `config.json`

## Key Files Modified

- `cody_state.py` - Added `llm_response`, `diff_extracted`, `logs_dir` fields
- `apply_diff.py` - Added detailed diagnostics and file saving
- `clippy_agent.py` - Added logging of API calls and responses
- `run_clippy.py` - Added warning/error counting and logging
- `main.py` - Enhanced final output with log directory info

## Running with Diagnostics

```bash
python .\cody-graph\main.py
```

Monitor the console for `[DIAG]` messages, then:
1. Review the log files in `.cody_logs/`
2. Use the diagnostic information to adjust prompts or configuration
3. Re-run to validate fixes

## Next Steps for Analysis

1. **Enable verbose git apply**: Modify `apply_diff.py` to add `-v` flag
2. **Add diff validation**: Check if patch is valid unified diff format before applying
3. **Save state snapshots**: Persist the entire CodyState after each step
4. **Model telemetry**: Track which models work best for different warning types
