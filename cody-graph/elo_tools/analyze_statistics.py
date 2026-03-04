#!/usr/bin/env python3
"""
ELO Gain: Statistical Analysis

Analyzes gauntlet match results (PGN) to calculate:
- ELO difference (ΔElo) between candidate and stable engines
- Bayes error bar (confidence interval)
- Statistical significance of the improvement

Supports two analysis modes:
1. cutechess-cli Bayes ELO calculator (if available)
2. Pure Python Bayesian analysis (with scipy/numpy)

Usage:
    python elo_tools/analyze_statistics.py \
        --pgn <gauntlet_result.pgn> \
        --candidate-name "Cody Candidate" \
        --stable-name "Cody Stable"

TODO: Implement full statistical analysis and Bayesian ELO calculation.
"""

import argparse
import json
import subprocess
from pathlib import Path
from typing import Dict, Tuple, Optional
import sys

# Optional imports for Bayesian analysis
try:
    import numpy as np
    from scipy import stats
    HAS_SCIPY = True
except ImportError:
    HAS_SCIPY = False


class BayesianELOAnalyzer:
    """Pure Python Bayesian ELO analyzer."""
    
    @staticmethod
    def calculate_elo_difference(
        candidate_score: float,
        stable_score: float,
        game_count: int,
    ) -> Tuple[float, float]:
        """
        Calculate ELO difference with Bayes error bar using beta-binomial conjugate prior.
        
        Args:
            candidate_score: Score as fraction [0, 1] (wins + 0.5*draws) / games
            stable_score: Score as fraction [0, 1]
            game_count: Total number of games played
        
        Returns:
            (elo_difference, error_bar)
        
        TODO: Implement proper Bayesian analysis with:
        - Beta-binomial conjugate prior
        - MCMC sampling for posterior distribution
        - Credible interval calculation
        """
        if game_count == 0:
            return 0.0, 0.0
        
        # Placeholder: Simple approximation
        # Real implementation uses Bayesian framework
        
        # Win rate difference
        delta = candidate_score - stable_score
        
        # Standard error for proportions
        # SE = sqrt(p1(1-p1)/n1 + p2(1-p2)/n2)
        se = np.sqrt(
            (candidate_score * (1 - candidate_score) +
             stable_score * (1 - stable_score)) / game_count
        ) if HAS_SCIPY else 0.0
        
        # Convert win rate to ELO (approximation)
        # ELO_diff ≈ 400 * log10(win_rate / loss_rate)
        if candidate_score > 0 and stable_score > 0:
            elo_diff = 400 * np.log10(candidate_score / stable_score) if HAS_SCIPY else 0.0
            # Error bar (approximate)
            error_bar = 400 * np.log10(np.e) * se if HAS_SCIPY else 0.0
        else:
            elo_diff = 0.0
            error_bar = 0.0
        
        return elo_diff, error_bar

def parse_pgn_for_scores(pgn_path: Path, candidate_name: str, stable_name: str) -> Dict:
    """
    Parse PGN file to extract match statistics.
    
    TODO: Implement full PGN parser to:
    - Identify which engine is which (white/black reversals)
    - Count wins, losses, draws
    - Calculate score percentages
    - Extract opening information
    
    For now, return placeholder structure.
    """
    print(f"[analyzer] Parsing PGN: {pgn_path}")
    
    if not pgn_path.exists():
        print(f"[analyzer] Warning: PGN file not found: {pgn_path}")
        return {
            "candidate_wins": 0,
            "stable_wins": 0,
            "draws": 0,
            "total_games": 0,
            "candidate_score": 0.0,
            "stable_score": 0.0,
        }
    
    # TODO: Implement robust PGN parsing
    # - Handle multiple games
    # - Parse Result field: 1-0, 0-1, 1/2-1/2
    # - Track white/black alternation
    # - Calculate aggregate scores
    
    # Placeholder
    return {
        "candidate_wins": 0,
        "stable_wins": 0,
        "draws": 0,
        "total_games": 0,
        "candidate_score": 0.0,
        "stable_score": 0.0,
    }

def try_cutechess_analysis(pgn_path: Path) -> Optional[Dict]:
    """
    Attempt to use cutechess-cli for Bayes ELO calculation.
    
    Returns None if cutechess-cli is not available or fails.
    
    TODO: Implement cutechess-cli integration.
    """
    try:
        # Check if cutechess-cli is available
        result = subprocess.run(
            ["cutechess-cli", "-version"],
            capture_output=True,
            timeout=5,
        )
        if result.returncode != 0:
            return None
        
        # TODO: Call cutechess-cli with -bayes option
        # Example: cutechess-cli -pgn <file> -bayes
        
        print("[analyzer] cutechess-cli available (TODO: integrate)")
        return None
    except (FileNotFoundError, subprocess.TimeoutExpired):
        return None

def analyze_gauntlet(
    pgn_path: Path,
    candidate_name: str = "Cody Candidate",
    stable_name: str = "Cody Stable",
) -> Dict:
    """
    Analyze gauntlet results and compute ELO statistics.
    
    Returns:
        {
            "elo_difference": float,
            "elo_error_bar": float,
            "candidate_wins": int,
            "stable_wins": int,
            "draws": int,
            "total_games": int,
            "candidate_score_percent": float,
            "statistically_significant": bool,
        }
    
    TODO: Implement full analysis with proper Bayesian framework.
    """
    print(f"[analyzer] Starting statistical analysis")
    print(f"  Candidate: {candidate_name}")
    print(f"  Stable:    {stable_name}\n")
    
    # Step 1: Parse PGN
    scores = parse_pgn_for_scores(pgn_path, candidate_name, stable_name)
    
    # Step 2: Try cutechess-cli first
    cutechess_result = try_cutechess_analysis(pgn_path)
    if cutechess_result:
        print(f"[analyzer] Using cutechess-cli analysis")
        return cutechess_result
    
    # Step 3: Fall back to pure Python Bayesian analysis
    if HAS_SCIPY:
        print(f"[analyzer] Using scipy-based Bayesian analysis")
        candidate_score = scores.get("candidate_score", 0.0)
        stable_score = scores.get("stable_score", 0.0)
        game_count = scores.get("total_games", 0)
        
        elo_diff, error_bar = BayesianELOAnalyzer.calculate_elo_difference(
            candidate_score, stable_score, game_count
        )
    else:
        print(f"[analyzer] Warning: scipy not available, using placeholder analysis")
        elo_diff = 0.0
        error_bar = 0.0
    
    # Determine statistical significance (very rough heuristic)
    # Proper test: elo_diff > 1.96 * error_bar (95% confidence)
    is_significant = abs(elo_diff) > 1.96 * error_bar if error_bar > 0 else False
    
    result = {
        "elo_difference": round(elo_diff, 1),
        "elo_error_bar": round(error_bar, 1),
        "candidate_wins": scores.get("candidate_wins", 0),
        "stable_wins": scores.get("stable_wins", 0),
        "draws": scores.get("draws", 0),
        "total_games": scores.get("total_games", 0),
        "candidate_score_percent": round(scores.get("candidate_score", 0.0) * 100, 1),
        "statistically_significant": is_significant,
    }
    
    print(f"\n[analyzer] Analysis Results:")
    print(f"  ELO Difference: {result['elo_difference']:.1f} ± {result['elo_error_bar']:.1f}")
    print(f"  Candidate Score: {result['candidate_score_percent']:.1f}%")
    print(f"  Record: {result['candidate_wins']}W - {result['stable_wins']}L - {result['draws']}D")
    print(f"  Significant: {result['statistically_significant']}\n")
    
    return result

def main():
    parser = argparse.ArgumentParser(
        description="Analyze gauntlet match results for ELO gain"
    )
    parser.add_argument(
        "--pgn",
        required=True,
        help="Path to PGN file from gauntlet"
    )
    parser.add_argument(
        "--candidate-name",
        default="Candidate",
        help="Name of candidate engine in PGN"
    )
    parser.add_argument(
        "--stable-name",
        default="Stable",
        help="Name of stable engine in PGN"
    )
    
    args = parser.parse_args()
    pgn_path = Path(args.pgn)
    
    result = analyze_gauntlet(
        pgn_path,
        candidate_name=args.candidate_name,
        stable_name=args.stable_name,
    )
    
    print(json.dumps(result, indent=2))
    return 0

if __name__ == "__main__":
    sys.exit(main())
