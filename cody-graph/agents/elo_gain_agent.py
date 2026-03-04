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
import sys
from pathlib import Path
from typing import Optional
from openai import OpenAI
from state.cody_state import CodyState

# Add elo_tools to path for imports
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "elo_tools"))
from gauntlet_runner import run_gauntlet, GauntletResult
from version_manager import (
    get_version_string,
    copy_binary_with_version,
    copy_candidate_binary,
)

# Add tools to path for commit utility
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
from commit_util import commit_with_version_bump

DEFAULT_MAX_ELO_PHASE_ITERATIONS = 1  # Demo mode: just 1 iteration to show phases
DEFAULT_TARGET_ELO_SUCCESSES = 5     # Number of successful improvements to achieve
DEFAULT_GAUNTLET_GAME_COUNT = 50    # 50–100 games at fast time control

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
    print("[cody-graph] [ELO Gain] [1/5] Candidate Generation phase [NOT IMPLEMENTED]", flush=True)
    
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
    print("[cody-graph] [ELO Gain] [2/5] Compilation & Validation phase [NOT IMPLEMENTED]", flush=True)
    
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
    
    Runs a short match (50–200 games) at fast time control (10s + 0.1s increment)
    against the stable/previous version of Cody using SPRT.
    
    Generates a PGN file with all games for statistical analysis and later review.
    """
    print("[cody-graph] [ELO Gain] [3/5] Running Gauntlet matches", flush=True)
    
    repo_path = state.get("repo_path", ".")
    game_count = state.get("elo_gauntlet_games", DEFAULT_GAUNTLET_GAME_COUNT)
    engines_dir = state.get("elo_engines_dir", r"C:\chess\Engines")
    
    # Build candidate binary path
    candidate_binary = os.path.join(repo_path, "target", "release", "cody.exe")
    
    # Run the gauntlet using cutechess-cli with SPRT
    try:
        result: GauntletResult = run_gauntlet(
            candidate_binary=candidate_binary,
            engines_dir=engines_dir,
            game_count=game_count,
            time_control="10+0.1",
        )
        
        # Store results in state
        state["elo_gauntlet_result"] = result.to_dict()
        state["elo_gauntlet_pgn"] = result.pgn_file
        state["elo_worst_fail_pgn"] = result.worst_fail_pgn
        state["elo_match_stats"] = {
            "games": result.games_played,
            "candidate_wins": result.candidate_wins,
            "champion_wins": result.champion_wins,
            "draws": result.draws,
            "candidate_score_percent": result.candidate_score * 100,
        }
        
        # Handle different result scenarios
        if result.status == "illegal_move":
            print(
                "[cody-graph] [ELO Gain] 🛑 CRITICAL: Candidate made an illegal move!",
                flush=True
            )
            state["status"] = "illegal_move"
            state["elo_phase_stage"] = "revert"
            state["elo_phase_outcome"] = "illegal_move"
            return state
        
        elif result.status == "error":
            print(
                f"[cody-graph] [ELO Gain] ERROR: {result.error_message}",
                flush=True
            )
            state["status"] = "error"
            state["elo_phase_stage"] = "revert"
            return state
        
        # Normal flow - proceed to statistical check
        state["status"] = "ok"
        state["elo_phase_stage"] = "statistical_check"
        state["last_command"] = "gauntlet_match"
        
    except Exception as e:
        print(f"[cody-graph] [ELO Gain] Gauntlet ERROR: {e}", flush=True)
        state["status"] = "error"
        state["elo_phase_stage"] = "revert"
        state["elo_error_message"] = str(e)
    
    return state

def elo_gain_statistical_check(state: CodyState) -> CodyState:
    """
    PHASE 4: Statistical Analysis
    
    With SPRT, the statistical decision is already made by cutechess-cli.
    This phase extracts and interprets the SPRT result.
    
    SPRT Decision:
    - H1: Candidate is significantly better (+5 ELO with 95% confidence)
    - H0: Candidate is NOT better (rejected hypothesis)
    - None: Inconclusive (max games reached without decision)
    """
    print("[cody-graph] [ELO Gain] [4/5] Statistical Analysis", flush=True)
    
    gauntlet_result = state.get("elo_gauntlet_result", {})
    sprt_decision = gauntlet_result.get("sprt_decision")
    candidate_score = gauntlet_result.get("candidate_score_percent", 0.0)
    
    # SPRT provides the statistical test
    if sprt_decision == "H1":
        # Candidate passed: statistically significant improvement
        print("[cody-graph] [ELO Gain] ✓ SPRT: H1 accepted (candidate is better)", flush=True)
        elo_estimate = 5.0  # Minimum improvement per SPRT config
        state["elo_gain_value"] = elo_estimate
        state["elo_passed_sprt"] = True
    
    elif sprt_decision == "H0":
        # Candidate failed: no significant improvement
        print("[cody-graph] [ELO Gain] ✗ SPRT: H0 (candidate not better)", flush=True)
        elo_estimate = 0.0
        state["elo_gain_value"] = elo_estimate
        state["elo_passed_sprt"] = False
    
    else:
        # Inconclusive: estimate from win rate
        print("[cody-graph] [ELO Gain] ⚠ SPRT: Inconclusive (max games reached)", flush=True)
        # Rough ELO estimate from win percentage
        if candidate_score > 50:
            elo_estimate = (candidate_score - 50) * 7  # Rough conversion
        else:
            elo_estimate = 0.0
        state["elo_gain_value"] = elo_estimate
        state["elo_passed_sprt"] = False
    
    state["status"] = "ok"
    state["elo_phase_stage"] = "decision"
    state["last_command"] = "statistical_check"
    
    print(
        f"[cody-graph] [ELO Gain] Estimated ELO: {elo_estimate:+.1f} "
        f"(Win rate: {candidate_score:.1f}%)",
        flush=True
    )
    
    return state

def elo_gain_decision(state: CodyState) -> CodyState:
    """
    PHASE 5: Decision & Commit or Revert
    
    Logic:
    - If SPRT H1 (passed): 
      * Increment patch version
      * Copy binary to engines dir with version
      * Commit changes with version bump
      * Update baseline for next iteration
    - If SPRT H0 or failed:
      * Revert the candidate
      * Feed loss PGNs back to LLM for analysis of failure modes
      * Record in learning state for future iterations
    """
    print("[cody-graph] [ELO Gain] [5/5] Decision phase", flush=True)
    
    repo_path = state.get("repo_path", ".")
    passed_sprt = state.get("elo_passed_sprt", False)
    elo_gain = state.get("elo_gain_value", 0.0)
    engines_dir = state.get("elo_engines_dir", r"C:\chess\Engines")
    
    if passed_sprt:
        print(f"[cody-graph] [ELO Gain] ✓ SPRT passed — COMMITTING", flush=True)
        
        try:
            # 1. Get current version before commit (commit_util will increment it)
            current_version = get_version_string(os.path.join(repo_path, "engine", "Cargo.toml"))
            print(f"[cody-graph] [ELO Gain] Current version: {current_version}", flush=True)
            
            # 2. Commit changes with automatic version bump
            commit_message = f"ELO gain (+{elo_gain:.1f} ELO estimated)"
            success, new_version, error = commit_with_version_bump(
                repo_path=repo_path,
                commit_message=commit_message,
                phase="ELOGain",
                files_to_add=None  # Will add all modified files + Cargo.toml
            )
            
            if not success:
                raise Exception(f"Commit failed: {error}")
            
            print(f"[cody-graph] [ELO Gain] Version: {current_version} → {new_version}", flush=True)
            
            # 3. Copy binary to engines dir with new version
            candidate_binary = os.path.join(repo_path, "target", "release", "cody.exe")
            versioned_binary = copy_binary_with_version(
                candidate_binary,
                engines_dir,
                new_version
            )
            print(f"[cody-graph] [ELO Gain] Copied to {versioned_binary}", flush=True)
            
            # Track successful commit
            successful_commits = state.get("elo_successful_commits", 0) + 1
            state["elo_successful_commits"] = successful_commits
            
            state["status"] = "ok"
            state["elo_phase_outcome"] = "committed"
            state["elo_improvement_committed"] = elo_gain
            state["elo_committed_version"] = new_version
            
        except Exception as e:
            print(f"[cody-graph] [ELO Gain] ERROR during commit: {e}", flush=True)
            state["status"] = "error"
            state["elo_phase_outcome"] = "commit_failed"
            state["elo_error_message"] = str(e)
    
    else:
        print(
            f"[cody-graph] [ELO Gain] ✗ SPRT failed or inconclusive — REVERTING",
            flush=True
        )
        
        # TODO: Revert and analysis logic
        # - Revert working directory to pre-candidate state (if needed)
        # - Analyze loss PGNs:
        #   * Parse PGN games where candidate lost
        #   * Extract position contexts, move sequences
        #   * Feed failures to LLM as context for next iteration
        # - Store analysis in a side object for learning
        
        # For now, just mark as reverted
        worst_fail = state.get("elo_worst_fail_pgn")
        if worst_fail:
            print(f"[cody-graph] [ELO Gain] Worst failure saved: {worst_fail}", flush=True)
        
        state["status"] = "ok"
        state["elo_phase_outcome"] = "reverted"
        state["elo_failure_analysis"] = "TODO: Analyze failure games and feed to LLM"
    
    state["last_command"] = "decision"
    state["elo_phase_stage"] = "complete"
    
    return state


def elo_gain_agent(state: CodyState) -> CodyState:
    """
    Main ELO Gain Orchestration
    
    Routes through the 5-phase loop until N successful improvements achieved
    or max iterations reached. Default target: N=5 successful commits.
    """
    if state.get("elo_iterations", 0) == 0:
        # First call: initialize tracking
        print("[cody-graph] [ELO Gain] Starting ELO Gain phase orchestration", flush=True)
    
    repo_path = state.get("repo_path", ".")
    iteration = state.get("elo_iterations", 0)
    successful_commits = state.get("elo_successful_commits", 0)
    target_successes = state.get("elo_target_successes", DEFAULT_TARGET_ELO_SUCCESSES)
    max_iterations = state.get("elo_max_iterations", DEFAULT_MAX_ELO_PHASE_ITERATIONS)
    
    # Main orchestration loop - runs through all stages in a single execution
    while True:
        # Check if we've reached the success target
        if successful_commits >= target_successes:
            print(
                f"[cody-graph] [ELO Gain] ✓ Achieved {successful_commits} successful improvements "
                f"(target: {target_successes}), ending ELO phase",
                flush=True
            )
            state["status"] = "ok"
            return state
        
        # Check iteration limit
        if iteration >= max_iterations:
            print(
                f"[cody-graph] [ELO Gain] Reached max iterations ({max_iterations}). "
                f"Completed {successful_commits}/{target_successes} target improvements. "
                "Ending ELO phase.",
                flush=True
            )
            state["status"] = "ok"
            return state
        
        # Route through stages
        stage = state.get("elo_phase_stage", "candidate_generation")
        
        if stage == "candidate_generation":
            state = elo_gain_candidate_generation(state)
        elif stage == "compilation":
            state = elo_gain_compilation_check(state)
        elif stage == "gauntlet":
            state = elo_gain_gauntlet_match(state)
        elif stage == "statistical_check":
            state = elo_gain_statistical_check(state)
        elif stage == "decision":
            state = elo_gain_decision(state)
        elif stage == "complete":
            # Increment iteration counter and prepare for next iteration
            iteration = state.get("elo_iterations", 0) + 1
            state["elo_iterations"] = iteration
            successful_commits = state.get("elo_successful_commits", 0)
            
            # Check if we should continue
            if successful_commits >= target_successes or iteration >= max_iterations:
                # Loop will check conditions at top of next iteration
                continue
            
            # Reset for next iteration
            state["elo_phase_stage"] = "candidate_generation"
            print(
                f"[cody-graph] [ELO Gain] Iteration {iteration} starting "
                f"({successful_commits}/{target_successes} successes)",
                flush=True
            )
            # Loop continues to next iteration
        else:
            print(f"[cody-graph] [ELO Gain] Unknown stage: {stage}", flush=True)
            state["status"] = "error"
            return state
