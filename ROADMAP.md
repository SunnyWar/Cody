# Cody Roadmap

## Current Status (March 2026)
- Project state: MVP engine complete, orchestration-driven improvement active.
- Core engine: Bitboard movegen, negamax alpha-beta, quiescence, TT, UCI, perft support.
- Automation: Multi-phase orchestration and diagnostics are operational.

## Active Priorities

### 1. Refactoring Phase
- Status: Infrastructure ready, not fully finalized.
- Goal: Improve maintainability and structure with safe, reviewable diffs.
- Next: Finalize prompts, acceptance criteria, and stop conditions.

### 2. Performance Phase
- Status: Operational with strategy framework.
- Goal: Improve hot-path speed while preserving correctness.
- Next: Tighten benchmark baselines and regression checks for each accepted optimization.

### 3. UCI Features Phase
- Status: Scaffolding ready, backlog pending.
- Goal: Expand tournament-grade UCI compatibility and option handling.
- Next: Prioritize missing commands and add protocol validation coverage.

### 4. ELO Gain Phase
- Status: Implemented and integrated.
- Goal: Convert candidate changes into measurable strength gains.
- Current loop: Candidate -> Compile -> Gauntlet -> Statistical Check -> Decision.
- Next: Continue iterative tuning and reduce false-positive candidates.

## Completed Milestones

### Engine MVP
- UCI protocol handling, board representation, pseudo-legal + legality validation.
- Negamax with alpha-beta, quiescence search, TT, time management.
- FEN parsing and move notation support.

### Infrastructure
- Cargo workspace split (`bitboard`, `engine`).
- Fixed-block arena model and type-safe search primitives.
- Unit/integration testing and criterion benchmarking.

### Orchestration
- Clippy phase automation with rollback safety.
- Multi-phase routing and state persistence.
- Timestamped diagnostics and traceability in `.cody_logs/`.
- ELO loop integration with version-management workflow.

## Quality Gates
- Build: `cargo build --release`
- Tests: `cargo test`, `cargo test -p bitboard`, `cargo test -p engine`
- Benchmarks: `cargo bench -p engine`
- Movegen validation: `cargo run --release -p engine -- perft 5`

## Constraints To Preserve
- Keep hot paths allocation-free where designed.
- Preserve separation of concerns: `bitboard` for rules, `engine` for search/UCI.
- Maintain semantic newtypes and explicit cast style for performance-sensitive code.

## References
- Overview and setup: `README.md`
- Command quick reference: `QUICKREF.md`
- Phase behavior: `cody-graph/PHASES.md`
- Troubleshooting/logs: `cody-graph/DIAGNOSTICS.md`
- ELO architecture: `cody-graph/ELO_GAIN_PHASE.md`
- Version policy: `cody-graph/VERSION_MANAGEMENT.md`
- Design notes: `architecture.md`
