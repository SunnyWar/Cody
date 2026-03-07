# Consolidated TODO

This file consolidates open TODO/checklist items from project documentation.
Completed entries were intentionally excluded.

## Hardware Optimization
Source: `HARDWARE_OPTIMIZATION_GUIDE.md`

- Add `target-cpu=znver3` in `Cargo.toml` profile.
- Implement move generation cache-line alignment.
- Verify SEE thresholds are optimal.
- Benchmark with `cargo bench --release`.
- Fine-tune aspiration window for current eval.
- Optimize per-thread arena usage.
- Profile with flamegraph to identify remaining hotspots.

## Cody Graph Version Management
Source: `cody-graph/VERSION_MANAGEMENT.md`

- Update Clippy Agent commit flow implementation notes.
- Implement Refactoring Agent with `commit_util.py`.
- Implement Performance Agent with `commit_util.py`.
- Implement UCI Features Agent with `commit_util.py`.

## ELO Gain Phase
Source: `cody-graph/ELO_GAIN_PHASE.md`

- Integrate with OpenAI/Anthropic LLM.
- Provide engine code context.
- Generate and validate diff format.
- Store candidate in state.
- Implement `validate_compilation.py` with timeout handling.
- Parse perft output and compare against known node counts.
- Validate in release mode.
- Return structured pass/fail results.
- Integrate with `cutechess-cli` or equivalent tournament runner.
- Build and locate both binaries (candidate and stable).
- Configure UCI engine settings.
- Run match with time control and match parameters.
- Parse live/final output statistics.
- Write PGN to disk.
- Handle timeouts and crashes gracefully.
- Parse PGN results and metadata.
- Calculate score percentages.
- Implement Bayesian ELO calculation (scipy or NumPy).
- Add posterior sampling for confidence estimates.
- Optionally add MCMC for higher accuracy.
- Validate analysis output against `cutechess-cli` if available.
- Return structured output with confidence intervals.
- Implement git operations (`add`, `commit`, `tag`, branch update).
- Implement loss analysis from PGN games.
- Improve error handling (merge conflicts, dirty working directory).
- Persist updated baseline state in `orchestrator_state.json`.
- Implement full `gauntlet_runner.py`.
- Implement full `analyze_statistics.py`.
- Implement full `commit_or_revert.py`.
- Feed failure analysis back to LLM for next iteration.
- Build stronger context prompts from failure reasons.
- Add integration tests for full ELO loop.
- Benchmark progress from current baseline to target ELO.
- Optimize gauntlet runtime.
- Fine-tune statistical thresholds.

## Phase Framework
Source: `cody-graph/PHASES.md`

- Add per-phase iteration limits.
- Add conditional phase branching from results.
- Add phase-specific rollback strategies.
- Add parallel phase execution with conflict detection.
- Add phase result aggregation/reporting.
- Add ELO failure-feedback loop to phase logic.
- Add adaptive game counts for close calls.

## Prompt Checklists (Process)
Sources:
- `.github/ai/prompts/refactoring_execution.md`
- `.github/ai/prompts/features_execution.md`
- `.github/ai/prompts/performance_execution.md`

- Ensure no new heap allocations in hot paths.
- Keep public API changes backward compatible, or mark breaking changes clearly.
- Respect crate separation between `bitboard` and `engine`.
- Keep changes testable with existing suite.
- Ensure feature implementation matches chess theory.
- Ensure tests pass (`cargo test`) and add new tests where needed.
- Ensure release build succeeds (`cargo build --release`).
- Ensure benchmarks run cleanly (`cargo bench -p engine`).
- Ensure documentation updates accompany feature changes.
- Measure or estimate ELO impact.
- Prevent regressions in existing functionality.
- Measure and document performance impact (>= 5% or architectural justification).
- Document safety for any `unsafe` code.
- Keep perft results unchanged.
- Preserve readability and maintainability.
