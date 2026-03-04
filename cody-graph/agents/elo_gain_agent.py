"""
ELO Gain Agent - Orchestrates the chess engine improvement loop.

This agent manages a sophisticated multi-step process:
1. Candidate Generation: LLM proposes chess-related improvements
2. Compilation: Verify code builds and passes perft tests
3. Gauntlet: Run matches against the stable version
4. Statistical Check: Calculate ELO difference with error bars
5. Decision: Commit improvements or revert and analyze losses
"""

import json
import os
import subprocess
from pathlib import Path
from typing import Optional
from openai import OpenAI
from state.cody_state import CodyState

DEFAULT_MAX_ELO_PHASE_ITERATIONS = 10
DEFAULT_GAUNTLET_GAME_COUNT = 50  # 50–100 games at fast time control

def _load_config(repo_path: str) -> dict:
    """Load configuration from cody-agent/config.json."""
    config_override = os.environ.get("CODY_CONFIG_PATH")
    if config_override:
        config_path = Path(config_override)
    else:
        config_path = Path(repo_path) / "cody-agent" / "config.json"
    if not config_path.exists():
        return {}
    try:
        return json.loads(config_path.read_text(encoding="utf-8"))
    except Exception:
        return {}

def _select_model(config: dict, phase: str = "ELOGain") -> str:
    """Select the appropriate model for ELO gain improvements."""
    models = config.get("models", {}) if isinstance(config, dict) else {}
    return models.get(phase) or config.get("model") or "gpt-4o"

def elo_gain_candidate_generation(state: CodyState) -> CodyState:
    """
    PHASE 1: Candidate Generation
    
    The LLM proposes a chess-specific improvement (e.g., Null Move Pruning,
    better move ordering, evaluation tweaks, etc.).
    
    TODO: Implement full candidate generation with LLM analysis.
    """
    print("[cody-graph] [ELO Gain] Starting Candidate Generation phase", flush=True)
    
    repo_path = state.get("repo_path", ".")
    config = _load_config(repo_path)
    model = _select_model(config)
    
    # TODO: Placeholder for candidate analysis
    # - Read current engine code structure
    # - Analyze recent test results / weaknesses
    # - Prompt LLM with engine architecture and chess concepts
    # - Generate diff with proposed improvement
    
    state["status"] = "ok"
    state["elo_phase_stage"] = "compilation"
    state["elo_proposed_candidate"] = "PLACEHOLDER: Proposed improvement description"
    state["last_command"] = "candidate_generation"
    
    return state

def elo_gain_compilation_check(state: CodyState) -> CodyState:
    """
    PHASE 2: Compilation & Validation
    
    Verifies that the proposed code:
    - Builds successfully (cargo build --release)
    - Passes basic perft tests (move generation correctness)
    - Does not introduce clippy warnings
    
    TODO: Implement full compilation and perft validation.
    """
    print("[cody-graph] [ELO Gain] Compilation & Validation phase", flush=True)
    
    repo_path = state.get("repo_path", ".")
    
    # TODO: Call compilation validation script
    # This should:
    # 1. Run: cargo build --release
    # 2. Run: cargo run --release -p engine -- perft 5
    # 3. Run: cargo clippy --all-targets
    # 4. Report any failures
    
    compilation_ok = True  # TODO: Set based on actual build result
    
    if not compilation_ok:
        print("[cody-graph] [ELO Gain] Compilation failed, reverting candidate", flush=True)
        state["status"] = "compilation_failed"
        state["elo_phase_stage"] = "revert"
        state["last_command"] = "compilation_check"
        return state
    
    state["status"] = "ok"
    state["elo_phase_stage"] = "gauntlet"
    state["last_command"] = "compilation_check"
    
    return state

def elo_gain_gauntlet_match(state: CodyState) -> CodyState:
    """
    PHASE 3: The Gauntlet
    
    Runs a short match (50–100 games) at fast time control (10s + 0.1s increment)
    against the stable/previous version of Cody.
    
    Generates a PGN file with all games for statistical analysis and later review.
    
    TODO: Implement full gauntlet runner using cutechess-cli or similar.
    """
    print("[cody-graph] [ELO Gain] Running Gauntlet matches", flush=True)
    
    repo_path = state.get("repo_path", ".")
    game_count = state.get("elo_gauntlet_games", DEFAULT_GAUNTLET_GAME_COUNT)
    
    # TODO: Call gauntlet runner script
    # - Ensure stable binary exists (build from stable branch or tag)
    # - Run gauntlet_runner.py with:
    #   - Current candidate vs. stable
    #   - Game count (default 50)
    #   - Time control (10s + 0.1s increment, or from config)
    # - Capture PGN output
    # - Store match statistics (wins, losses, draws, score %)
    
    state["status"] = "ok"
    state["elo_phase_stage"] = "statistical_check"
    state["elo_gauntlet_pgn"] = "PLACEHOLDER: PGN file path"
    state["elo_match_stats"] = {
        "games": game_count,
        "candidate_wins": 0,  # TODO: Parse from gauntlet result
        "stable_wins": 0,     # TODO: Parse from gauntlet result
        "draws": 0,           # TODO: Parse from gauntlet result
        "candidate_score_percent": 0.0,  # TODO: Calculate
    }
    state["last_command"] = "gauntlet_match"
    
    return state

def elo_gain_statistical_check(state: CodyState) -> CodyState:
    """
    PHASE 4: Statistical Analysis
    
    Uses cutechess-cli or Bayesian analysis to:
    - Calculate ELO difference (ΔElo)
    - Compute Bayes error bar (confidence interval)
    - Determine if the improvement is statistically significant
    
    TODO: Implement statistical analyzer.
    """
    print("[cody-graph] [ELO Gain] Statistical Analysis", flush=True)
    
    repo_path = state.get("repo_path", ".")
    pgn_path = state.get("elo_gauntlet_pgn")
    
    # TODO: Call statistical analyzer script
    # - Input: PGN file from gauntlet
    # - Tool options:
    #   a. Call cutechess-cli's Bayes ELO calculator
    #   b. Implement Bayesian analysis (scipy/numpy)
    # - Output: ELO difference, error bar, statistical significance
    
    elo_gain = 0.0  # TODO: Calculate from match result
    error_bar = 0.0  # TODO: Calculate Bayes error bar
    
    state["status"] = "ok"
    state["elo_phase_stage"] = "decision"
    state["elo_gain_value"] = elo_gain
    state["elo_error_bar"] = error_bar
    state["last_command"] = "statistical_check"
    
    print(
        f"[cody-graph] [ELO Gain] Statistical result: {elo_gain:.1f} ± {error_bar:.1f} ELO",
        flush=True
    )
    
    return state

def elo_gain_decision(state: CodyState) -> CodyState:
    """
    PHASE 5: Decision & Commit or Revert
    
    Logic:
    - If ELO Gain > 0 (statistically acceptable):
      * Commit the change to main branch (or stable branch)
      * Update baseline for next iteration
    - If ELO Gain <= 0:
      * Revert the candidate
      * Feed loss PGNs back to LLM for analysis of failure modes
      * Record in learning state for future iterations
    
    TODO: Implement decision logic and commit/revert handling.
    """
    print("[cody-graph] [ELO Gain] Decision phase", flush=True)
    
    repo_path = state.get("repo_path", ".")
    elo_gain = state.get("elo_gain_value", 0.0)
    pgn_path = state.get("elo_gauntlet_pgn")
    
    decision_threshold = 0.0  # Gain must be > 0 to commit
    
    if elo_gain > decision_threshold:
        print(f"[cody-graph] [ELO Gain] ✓ ELO gain {elo_gain:.1f} — COMMITTING", flush=True)
        
        # TODO: Commit logic
        # - Stage all modified files in bitboard/ and engine/
        # - Create commit with message: "ELOGain: [description] (+{elo_gain:.1f} ELO)"
        # - Update 'stable' branch pointer (or merge to main)
        # - Tag with version
        
        state["status"] = "ok"
        state["elo_phase_outcome"] = "committed"
        state["elo_improvement_committed"] = elo_gain
        
    else:
        print(
            f"[cody-graph] [ELO Gain] ✗ ELO gain {elo_gain:.1f} <= 0 — REVERTING",
            flush=True
        )
        
        # TODO: Revert and analysis logic
        # - Revert working directory to pre-candidate state
        # - Analyze loss PGNs:
        #   * Parse PGN games where candidate lost
        #   * Extract position contexts, move sequences
        #   * Feed failures to LLM as context for next iteration
        # - Store analysis in a side object for learning
        
        state["status"] = "ok"
        state["elo_phase_outcome"] = "reverted"
        state["elo_failure_analysis"] = "PLACEHOLDER: Analysis of why candidate failed"
    
    state["last_command"] = "decision"
    state["elo_phase_stage"] = "complete"
    
    return state

def elo_gain_agent(state: CodyState) -> CodyState:
    """
    Main ELO Gain Orchestration
    
    Routes through the 5-phase loop until max iterations reached or
    stable performance achieved.
    """
    print("[cody-graph] [ELO Gain] Starting ELO Gain phase orchestration", flush=True)
    
    repo_path = state.get("repo_path", ".")
    iteration = state.get("elo_iterations", 0)
    max_iterations = state.get("elo_max_iterations", DEFAULT_MAX_ELO_PHASE_ITERATIONS)
    
    # Check iteration limit
    if iteration >= max_iterations:
        print(
            f"[cody-graph] [ELO Gain] Reached max iterations ({max_iterations}), "
            "ending ELO phase",
            flush=True
        )
        state["status"] = "ok"
        return state
    
    # Route through stages
    stage = state.get("elo_phase_stage", "candidate_generation")
    
    if stage == "candidate_generation":
        return elo_gain_candidate_generation(state)
    elif stage == "compilation":
        return elo_gain_compilation_check(state)
    elif stage == "gauntlet":
        return elo_gain_gauntlet_match(state)
    elif stage == "statistical_check":
        return elo_gain_statistical_check(state)
    elif stage == "decision":
        return elo_gain_decision(state)
    elif stage == "complete":
        # Increment iteration counter and loop back
        state["elo_iterations"] = iteration + 1
        state["elo_phase_stage"] = "candidate_generation"
        print(f"[cody-graph] [ELO Gain] Iteration {iteration + 1} starting", flush=True)
        return elo_gain_agent(state)
    else:
        print(f"[cody-graph] [ELO Gain] Unknown stage: {stage}", flush=True)
        state["status"] = "error"
        return state
