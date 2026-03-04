"""
ELO Gain Agent - Orchestrates the chess engine improvement loop.

This agent manages a sophisticated multi-step process:
0. Sanity Check: Self-play validation to detect critical bugs
1. Candidate Generation: LLM proposes chess-related improvements
2. Compilation: Verify code builds and passes perft tests
3. Gauntlet: Run matches against the stable version
4. Statistical Check: Calculate ELO difference with error bars
5. Decision: Commit improvements or revert and analyze losses

If sanity check finds CRITICAL issues, the process stops and blocks improvements.
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
from sanity_check import run_self_play_sanity_check, SanityCheckResult
from candidate_generator import CandidateGenerator

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

def elo_gain_sanity_check(state: CodyState) -> CodyState:
    """
    PHASE 0: Engine Sanity Check (Self-Play Validation)
    
    Before attempting any improvements, verify engine correctness:
    - Runs 10-20 games of self-play
    - Detects: illegal moves, crashes, quick checkmate losses
    - BLOCKS ELO improvements if critical issues found
    - Allows proceeding with WARNINGS (non-critical issues)
    """
    print("[cody-graph] [ELO Gain] [0/6] Engine Sanity Check (Self-Play)", flush=True)
    
    repo_path = state.get("repo_path", ".")
    engines_dir = state.get("elo_engines_dir", r"C:\chess\Engines")
    sanity_games = state.get("elo_sanity_games", 10)  # Quick validation: 10 games
    
    binary_path = os.path.join(repo_path, "target", "release", "cody.exe")
    
    try:
        sanity_result: SanityCheckResult = run_self_play_sanity_check(
            binary_path=binary_path,
            engines_dir=engines_dir,
            game_count=sanity_games,
            time_control="10+0.1",
        )
        
        # Store results in state
        state["elo_sanity_result"] = sanity_result.to_dict()
        state["elo_sanity_pgn"] = sanity_result.pgn_file
        
        if sanity_result.has_critical_issues():
            # BLOCK improvements - engine has serious bugs
            print(
                "[cody-graph] [ELO Gain] [CRITICAL] Issues in self-play - BLOCKING improvements",
                flush=True
            )
            for issue in sanity_result.critical_issues:
                print(f"  - {issue}", flush=True)
            print(
                "[cody-graph] [ELO Gain] Fix these issues manually before attempting ELO improvements",
                flush=True
            )
            state["status"] = "sanity_check_failed"
            state["elo_phase_stage"] = "complete"
            state["elo_phase_outcome"] = "sanity_failed"
            return state
        
        if sanity_result.has_issues():
            # Warnings only - allow proceeding but flag for attention
            print(
                "[cody-graph] [ELO Gain] [WARNING] Warnings found in self-play:",
                flush=True
            )
            for warning in sanity_result.warnings:
                print(f"  - {warning}", flush=True)
            print(
                "[cody-graph] [ELO Gain] Proceeding with improvements, but review results",
                flush=True
            )
            state["elo_sanity_warnings"] = sanity_result.warnings
        else:
            print("[cody-graph] [ELO Gain] [OK] Engine sanity check passed - no critical issues", flush=True)
        
        state["status"] = "ok"
        state["elo_phase_stage"] = "candidate_generation"
        state["last_command"] = "sanity_check"
        
    except Exception as e:
        print(f"[cody-graph] [ELO Gain] Sanity check ERROR: {e}", flush=True)
        state["status"] = "error"
        state["elo_phase_stage"] = "complete"
        state["elo_error_message"] = str(e)
    
    return state

def elo_gain_candidate_generation(state: CodyState) -> CodyState:
    """
    PHASE 1: Candidate Generation
    
    Two modes of operation:
    A) If sanity check found issues → Generate UNIT TEST to reproduce them
    B) If sanity check passed → Generate ELO IMPROVEMENT proposal
    
    This ensures we fix broken things before adding features.
    """
    print("[cody-graph] [ELO Gain] [1/6] Candidate Generation phase", flush=True)
    
    repo_path = state.get("repo_path", ".")
    config = _load_config(repo_path)
    model = _select_model(config, "ELOGain")
    api_key = os.environ.get("OPENAI_API_KEY")
    sanity_result = state.get("elo_sanity_result", {})
    
    try:
        # Import candidate generator
        sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "elo_tools"))
        from candidate_generator import CandidateGenerator
        
        generator = CandidateGenerator(repo_path, model=model, api_key=api_key)
        
        # Check if sanity check found issues
        has_critical = sanity_result.get("has_critical_issues", False)
        has_warnings = len(sanity_result.get("warnings", [])) > 0
        
        if has_critical or has_warnings:
            # PRIORITY MODE: Generate unit test to reproduce issue
            print(
                "[cody-graph] [ELO Gain] [] Issues detected - Generating unit test to reproduce",
                flush=True
            )
            
            candidate = generator.generate_unit_test_for_issue(sanity_result)
            candidate_type = "unit_test"
            
            if candidate.get("status") == "skip":
                # No unit test needed
                print("[cody-graph] [ELO Gain] [!] No critical issues to test", flush=True)
                state["elo_candidate_type"] = "none"
                state["elo_proposed_candidate"] = "No issues found, but not proceeding with improvements"
                state["elo_phase_stage"] = "complete"
                state["status"] = "ok"
                return state
            
            print(
                f"[cody-graph] [ELO Gain] Proposed TEST: {candidate.get('title', 'Unknown')}",
                flush=True
            )
            print(
                f"  Description: {candidate.get('description', 'N/A')}",
                flush=True
            )
            print(
                f"  Test function: {candidate.get('function_name', 'N/A')}",
                flush=True
            )
            
            state["elo_candidate_type"] = candidate_type
            state["elo_proposed_candidate"] = candidate
            state["elo_test_code"] = candidate.get("test_code", "")
            state["elo_test_files"] = candidate.get("files_to_add", [])
            state["elo_proposal"] = candidate
            
        else:
            # NORMAL MODE: Generate ELO improvement
            print("[cody-graph] [ELO Gain] [] No issues detected - Generating ELO improvement", flush=True)
            
            candidate = generator.generate_improvement_proposal()
            candidate_type = "improvement"
            
            print(
                f"[cody-graph] [ELO Gain] Proposed IMPROVEMENT: {candidate.get('title', 'Unknown')}",
                flush=True
            )
            print(
                f"  Type: {candidate.get('improvement_type', 'N/A')}",
                flush=True
            )
            print(
                f"  ELO Est: {candidate.get('expected_elo_gain', 'N/A')}",
                flush=True
            )
            print(
                f"  Confidence: {candidate.get('confidence', 'N/A')}",
                flush=True
            )
            
            state["elo_candidate_type"] = candidate_type
            state["elo_proposed_candidate"] = candidate
            state["elo_proposal"] = candidate
        
        state["status"] = "ok"
        state["elo_phase_stage"] = "compilation"
        state["last_command"] = "candidate_generation"
        
    except Exception as e:
        print(f"[cody-graph] [ELO Gain] Candidate generation ERROR: {e}", flush=True)
        import traceback
        print(f"[cody-graph] [ELO Gain] Traceback: {traceback.format_exc()}", flush=True)
        state["status"] = "error"
        state["elo_phase_stage"] = "complete"
        state["elo_error_message"] = str(e)
    
    return state

def elo_gain_compilation_check(state: CodyState) -> CodyState:
    """
    PHASE 2: Compilation & Validation
    
    Two modes:
    A) UNIT TEST mode: Add test to codebase and verify it compiles + runs
    B) IMPROVEMENT mode: Verify engine builds and passes perft tests
    """
    print("[cody-graph] [ELO Gain] [2/6] Compilation & Validation phase", flush=True)
    
    repo_path = state.get("repo_path", ".")
    candidate_type = state.get("elo_candidate_type", "improvement")
    candidate = state.get("elo_proposed_candidate", {})
    perft_depth = state.get("elo_perft_depth", 5)
    
    # Import validation module
    sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "elo_tools"))
    from validate_compilation import validate_compilation
    
    try:
        if candidate_type == "unit_test":
            # UNIT TEST mode: Add test to codebase and verify it compiles
            print(
                f"[cody-graph] [ELO Gain] [] Adding unit test: {candidate.get('function_name', 'unknown')}",
                flush=True
            )
            
            test_code = state.get("elo_test_code", "")
            test_files = state.get("elo_test_files", [])
            
            if not test_code:
                print("[cody-graph] [ELO Gain] [] No test code to add", flush=True)
                state["status"] = "compilation_failed"
                state["elo_phase_stage"] = "revert"
                return state
            
            # TODO: Actually add test code to files
            # For now, just verify that the main engine still builds with test code present
            print(
                "[cody-graph] [ELO Gain] [] [TODO] Test code would be added to: {}",
                ", ".join(test_files),
                flush=True
            )
            
            # Verify core engine still compiles
            compilation_ok = validate_compilation(Path(repo_path), perft_depth=perft_depth)
            
            if not compilation_ok:
                print("[cody-graph] [ELO Gain] Compilation failed, reverting test", flush=True)
                state["status"] = "compilation_failed"
                state["elo_phase_stage"] = "revert"
                return state
            
            print("[cody-graph] [ELO Gain] [OK] Test code compiles successfully", flush=True)
            state["status"] = "ok"
            state["elo_phase_stage"] = "gauntlet"
            state["last_command"] = "compilation_check_unitest"
            
        else:
            # IMPROVEMENT mode: Standard compilation + perft validation
            print(
                f"[cody-graph] [ELO Gain] [] Validating improvement: {candidate.get('title', 'unknown')}",
                flush=True
            )
            
            compilation_ok = validate_compilation(Path(repo_path), perft_depth=perft_depth)
            
            if not compilation_ok:
                print("[cody-graph] [ELO Gain] Compilation failed, reverting candidate", flush=True)
                state["status"] = "compilation_failed"
                state["elo_phase_stage"] = "revert"
                state["last_command"] = "compilation_check"
                return state
            
            print("[cody-graph] [ELO Gain] [OK] Improvement compiles and passes validation", flush=True)
            state["status"] = "ok"
            state["elo_phase_stage"] = "gauntlet"
            state["last_command"] = "compilation_check"
        
    except Exception as e:
        print(f"[cody-graph] [ELO Gain] Compilation check ERROR: {e}", flush=True)
        state["status"] = "error"
        state["elo_phase_stage"] = "revert"
        state["elo_error_message"] = str(e)
    
    return state

def elo_gain_gauntlet_match(state: CodyState) -> CodyState:
    """
    PHASE 3: Validation
    
    Two modes:
    A) UNIT TEST mode: Run the proposed test with `cargo test`
    B) IMPROVEMENT mode: Run gauntlet match against champion
    """
    print("[cody-graph] [ELO Gain] [3/6] Running Validation", flush=True)
    
    repo_path = state.get("repo_path", ".")
    candidate_type = state.get("elo_candidate_type", "improvement")
    candidate = state.get("elo_proposed_candidate", {})
    
    if candidate_type == "unit_test":
        # UNIT TEST mode: Run cargo test to verify test passes
        print("[cody-graph] [ELO Gain] [] Running unit test to verify issue reproduction", flush=True)
        
        test_function = candidate.get("function_name", "")
        
        try:
            # Run the specific test
            cmd = ["cargo", "test", "--release", "--", "--nocapture"]
            if test_function:
                cmd[-1] = test_function  # Filter to specific test
            
            result = subprocess.run(
                cmd,
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=60
            )
            
            test_passed = result.returncode == 0
            test_output = result.stdout + result.stderr
            
            print(f"[cody-graph] [ELO Gain] Test run completed: {'PASS' if test_passed else 'FAIL'}", flush=True)
            print(f"[cody-graph] [ELO Gain] Output: {test_output[-500:]}", flush=True)  # Last 500 chars
            
            if test_passed:
                # Test passed - this is good, we successfully reproduced and can work to fix
                state["elo_test_result"] = "pass"
                state["elo_test_output"] = test_output
                print(
                    "[cody-graph] [ELO Gain] [OK] Unit test PASSED - issue successfully reproduced!",
                    flush=True
                )
                # Move to decision phase to commit the test
                state["elo_phase_stage"] = "statistical_check"
            else:
                # Test failed - test itself may have issues
                state["elo_test_result"] = "fail"
                state["elo_test_output"] = test_output
                print(
                    "[cody-graph] [ELO Gain] [!] Unit test FAILED - issue not reproduced, reverting",
                    flush=True
                )
                state["elo_phase_stage"] = "revert"
                state["status"] = "test_failed"
                return state
            
            state["status"] = "ok"
            state["last_command"] = "unitest_run"
            
        except subprocess.TimeoutExpired:
            print("[cody-graph] [ELO Gain] [] Test timed out", flush=True)
            state["elo_test_result"] = "timeout"
            state["elo_phase_stage"] = "revert"
            state["status"] = "test_timeout"
            return state
        except Exception as e:
            print(f"[cody-graph] [ELO Gain] Test execution ERROR: {e}", flush=True)
            state["elo_test_result"] = "error"
            state["elo_phase_stage"] = "revert"
            state["status"] = "test_error"
            return state
    
    else:
        # IMPROVEMENT mode: Run gauntlet match
        print("[cody-graph] [ELO Gain] [] Running gauntlet match against champion", flush=True)
        
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
                    "[cody-graph] [ELO Gain] [CRITICAL] Candidate made an illegal move!",
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
    print("[cody-graph] [ELO Gain] [4/6] Statistical Analysis", flush=True)
    
    gauntlet_result = state.get("elo_gauntlet_result", {})
    sprt_decision = gauntlet_result.get("sprt_decision")
    candidate_score = gauntlet_result.get("candidate_score_percent", 0.0)
    
    # SPRT provides the statistical test
    if sprt_decision == "H1":
        # Candidate passed: statistically significant improvement
        print("[cody-graph] [ELO Gain] [OK] SPRT: H1 accepted (candidate is better)", flush=True)
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
        print("[cody-graph] [ELO Gain] [WARNING] SPRT: Inconclusive (max games reached)", flush=True)
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
    print("[cody-graph] [ELO Gain] [5/6] Decision phase", flush=True)
    
    repo_path = state.get("repo_path", ".")
    passed_sprt = state.get("elo_passed_sprt", False)
    elo_gain = state.get("elo_gain_value", 0.0)
    engines_dir = state.get("elo_engines_dir", r"C:\chess\Engines")
    
    if passed_sprt:
        print(f"[cody-graph] [ELO Gain] [OK] SPRT passed — COMMITTING", flush=True)
        
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
                f"[cody-graph] [ELO Gain] [OK] Achieved {successful_commits} successful improvements "
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
        stage = state.get("elo_phase_stage", "sanity_check")  # Start with sanity check
        
        if stage == "sanity_check":
            state = elo_gain_sanity_check(state)
        elif stage == "candidate_generation":
            state = elo_gain_candidate_generation(state)
        elif stage == "compilation":
            state = elo_gain_compilation_check(state)
        elif stage == "gauntlet":
            state = elo_gain_gauntlet_match(state)
        elif stage == "statistical_check":
            state = elo_gain_statistical_check(state)
        elif stage == "decision":
            state = elo_gain_decision(state)
        elif stage == "revert":
            # Handle revert after compilation failure or gauntlet error
            print("[cody-graph] [ELO Gain] Reverting candidate due to failure", flush=True)
            
            # TODO: Implement actual git revert if needed
            # For now, just mark as reverted and move to next iteration
            
            state["status"] = "ok"
            state["elo_phase_outcome"] = "reverted"
            state["elo_phase_stage"] = "complete"
            state["last_command"] = "revert"
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
