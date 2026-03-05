#!/usr/bin/env python3
r"""
Candidate Generation for ELO Gain Phase

Proposes chess-specific improvements via LLM analysis:
1. Reads engine architecture (move generation, evaluation, search)
2. Analyzes recent game failures (if available)
3. Generates improvement proposals (Null Move, Better Eval, Move Ordering, etc.)
4. Creates unified diff for application

Candidate improvements target:
- Search enhancements: null move pruning, LMR, time management
- Evaluation tweaks: piece-square tables, king safety, material balance
- Move ordering: killer moves, history heuristic, SEE ordering
- Position evaluation: mobility, pawn structure, tempo

Usage:
    python elo_tools/candidate_generator.py --repo-path D:\Cody
"""

import os
import json
import sys
from pathlib import Path
from typing import Optional, Dict, List
from textwrap import dedent

# Try to import OpenAI; gracefully fail if not available
try:
    from openai import OpenAI
    HAS_OPENAI = True
except ImportError:
    HAS_OPENAI = False
    print("[candidate_generator] Warning: OpenAI SDK not available, using placeholder mode")


class CandidateGenerator:
    """Generate improvement proposals for chess engine."""
    
    def __init__(self, repo_path: str, model: str = "o3-mini", api_key: Optional[str] = None):
        self.repo_path = Path(repo_path)
        self.model = model
        self.client = None
        
        if HAS_OPENAI:
            api_key = api_key or os.environ.get("OPENAI_API_KEY")
            if api_key:
                self.client = OpenAI(api_key=api_key)
    
    def read_repository_context(self) -> str:
        """Read key engine source files to understand architecture."""
        context_files = [
            "engine/src/search/core.rs",
            "engine/src/search/engine.rs",
            "engine/src/core/eval.rs",
            "engine/src/core/arena.rs",
            "bitboard/src/movegen/api.rs",
            "bitboard/src/position.rs",
            "engine/Cargo.toml",
        ]
        
        fragments = []
        
        for file in context_files:
            file_path = self.repo_path / file
            if file_path.exists():
                try:
                    content = file_path.read_text(encoding="utf-8")
                    # Limit to first 2000 chars to conserve tokens
                    limited = content[:2000]
                    fragments.append(f"=== {file} ===\n{limited}\n...")
                except Exception as e:
                    fragments.append(f"=== {file} === (read error: {e})")
            else:
                fragments.append(f"=== {file} === (not found)")
        
        return "\n\n".join(fragments)
    
    def read_recent_failures(self) -> str:
        """Read worst failure PGN if available."""
        failure_pgn = Path(r"C:\chess\Engines\worst_fail.pgn")
        
        if not failure_pgn.exists():
            return "No recent failure games recorded yet."
        
        try:
            content = failure_pgn.read_text(encoding="utf-8")
            return f"Recent Failure Game:\n{content[:1500]}"
        except Exception:
            return "Could not read failure PGN."
    
    def generate_improvement_proposal(self) -> Dict:
        """
        Generate chess-specific improvement proposal via LLM.
        
        Returns:
            Dict with keys: improvement_type, description, reasoning, diff_snippet, implementation_notes
        """
        repo_context = self.read_repository_context()
        recent_failures = self.read_recent_failures()
        
        prompt = dedent(f"""
        You are a chess engine optimization expert. Analyze the following Cody chess engine structure
        and propose ONE concrete, focused improvement to increase ELO strength.
        
        CURRENT ENGINE ARCHITECTURE:
        {repo_context}
        
        RECENT FAILURE ANALYSIS:
        {recent_failures}
        
        TASK:
        1. Identify ONE specific weakness or optimization opportunity
        2. Propose a concrete fix (search enhancement, evaluation adjustment, or move ordering improvement)
        3. Explain why this change would improve ELO
        4. Describe the implementation approach
        
        Choose from these common improvements:
        - Better move ordering (killer moves, history heuristic, MVV-LVA)
        - Evaluation tweaks (piece-square table adjustment, king safety, mobility bonus)
        - Search enhancements (late move pruning, null move pruning, LMR refinements)
        - Time management (better remaining time allocation)
        - Transposition table (larger size, replacement strategy)
        - Position analysis (positional factors, pawn structure evaluation)
        
        RESPONSE FORMAT (JSON):
        {{
            "improvement_type": "Type of improvement (e.g., 'move_ordering', 'evaluation', 'search')",
            "title": "Short title",
            "reasoning": "Why this improves ELO",
            "description": "Detailed explanation of the change",
            "implementation_approach": "How to implement it",
            "expected_elo_gain": "Estimated ELO gain (rough estimate)",
            "risk_level": "low/medium/high",
            "files_affected": ["list of files to modify"],
            "confidence": "high/medium/low"
        }}
        """)
        
        if not self.client:
            # Placeholder mode: return a safe, simple improvement
            return self._placeholder_improvement()
        
        try:
            response = self.client.chat.completions.create(
                model=self.model,
                messages=[
                    {
                        "role": "user",
                        "content": prompt
                    }
                ],

                max_completion_tokens=1000,
            )
            
            response_text = response.choices[0].message.content
            
            # Try to extract JSON from response
            try:
                # Find JSON block
                import re
                json_match = re.search(r'\{.*\}', response_text, re.DOTALL)
                if json_match:
                    proposal = json.loads(json_match.group())
                else:
                    proposal = json.loads(response_text)
            except json.JSONDecodeError:
                # If LLM didn't return valid JSON, create structured response
                proposal = {
                    "improvement_type": "analysis_note",
                    "title": "LLM Response (unparsed)",
                    "description": response_text[:500],
                    "reasoning": "Raw LLM output",
                    "implementation_approach": "Requires manual review",
                    "expected_elo_gain": "Unknown",
                    "risk_level": "medium",
                    "files_affected": [],
                    "confidence": "low"
                }
            
            return proposal
            
        except Exception as e:
            print(f"[candidate_generator] LLM Error: {e}")
            return self._placeholder_improvement()
    
    def generate_unit_test_for_issue(self, sanity_result: Dict) -> Dict:
        """
        When sanity check finds issues, generate a unit test OR integration test to reproduce them.
        
        Chooses test type based on issue scope:
        - UNIT TEST: Move generation bugs, specific position evaluation issues
        - INTEGRATION TEST: Game-level issues, search behavior, move ordering in real games
        
        Args:
            sanity_result: Dict from sanity_check.py containing issues found
        
        Returns:
            Dict with test proposal (test_type, description, test_code, etc.)
        """
        quick_losses = sanity_result.get("quick_losses", [])
        illegal_moves = sanity_result.get("illegal_moves", [])
        warnings = sanity_result.get("warnings", [])
        pgn_file = sanity_result.get("pgn_file")
        
        issues_found = []
        if illegal_moves:
            issues_found.append(f"CRITICAL: {len(illegal_moves)} illegal moves detected")
        if quick_losses:
            issues_found.append(f"WARNING: {len(quick_losses)} quick losses (potential eval issues)")
        if warnings:
            issues_found.extend(warnings)
        
        if not issues_found:
            return {
                "test_type": "none_needed",
                "title": "Engine is sound",
                "description": "Sanity check passed with no critical issues",
                "status": "skip"
            }
        
        # Determine test focus and type
        if illegal_moves:
            # Illegal moves = move generation bug = UNIT TEST
            test_focus = "reproduce_illegal_move"
            test_variant = "unit"
            issue_description = f"Illegal moves found: {illegal_moves[0] if illegal_moves else 'unknown'}"
        elif quick_losses:
            # Quick losses = evaluation or game-level bug = INTEGRATION TEST
            test_focus = "reproduce_bad_evaluation"
            test_variant = "integration"
            issue_description = f"Quick checkmate in < 10 moves: {quick_losses[0] if quick_losses else 'unknown'}"
        else:
            # General stability = INTEGRATION TEST
            test_focus = "general_stability"
            test_variant = "integration"
            issue_description = "General engine stability concerns"
        
        if test_variant == "unit":
            return self._generate_unit_test(test_focus, issue_description)
        else:
            return self._generate_integration_test(test_focus, issue_description, pgn_file)
    
    def _generate_unit_test(self, test_focus: str, issue_description: str) -> Dict:
        """Generate a unit test for isolated move generation or evaluation issues."""
        prompt = dedent(f"""
        You are a chess engine test engineer. A chess engine has been found to have the following issue:
        
        ISSUE: {issue_description}
        FOCUS: {test_focus}
        
        CONTEXT:
        - Engine: Cody chess engine (Rust-based, move generation heavy)
        - This is a UNIT TEST (not an integration test)
        - Tests isolated functionality (move gen, specific position eval)
        
        TASK:
        Create a MINIMAL Rust unit test that:
        1. Tests a SPECIFIC, ISOLATED chess position or move generation scenario
        2. Reproduces the issue in isolation
        3. Can be added to bitboard/src/lib.rs with #[cfg(test)]
        4. Runs quickly (< 100ms)
        
        RESPONSE (JSON):
        {{
            "test_type": "unit",
            "test_name": "test_name_here",
            "title": "Short test name",
            "description": "What the test checks",
            "chess_fen": "Starting FEN for the test (or 'startpos')",
            "test_code": "Complete Rust unit test function code",
            "explanation": "Why this test is important",
            "module": "bitboard",
            "confidence": "high/medium/low"
        }}
        """)
        
        if not self.client:
            return self._placeholder_unit_test(test_focus, issue_description)
        
        try:
            response = self.client.chat.completions.create(
                model=self.model,
                messages=[{"role": "user", "content": prompt}],
                max_completion_tokens=1500,
            )
            
            response_text = response.choices[0].message.content
            
            try:
                import re
                json_match = re.search(r'\{.*\}', response_text, re.DOTALL)
                if json_match:
                    test_proposal = json.loads(json_match.group())
                else:
                    test_proposal = json.loads(response_text)
            except json.JSONDecodeError:
                test_proposal = self._placeholder_unit_test(test_focus, issue_description)
            
            return test_proposal
            
        except Exception as e:
            print(f"[candidate_generator] LLM Error generating unit test: {e}")
            return self._placeholder_unit_test(test_focus, issue_description)
    
    def _generate_integration_test(self, test_focus: str, issue_description: str, pgn_file: Optional[str] = None) -> Dict:
        """Generate an integration test for game-level or search behavior issues."""
        prompt = dedent(f"""
        You are a chess engine test engineer. A chess engine has been found to have the following issue:
        
        ISSUE: {issue_description}
        FOCUS: {test_focus}
        
        CONTEXT:
        - Engine: Cody chess engine (Rust-based)
        - This is an INTEGRATION TEST (not a unit test)
        - Tests engine behavior across multiple moves/positions (search, eval, decision-making)
        - Can take longer to run (up to 5 seconds is OK)
        
        TASK:
        Create a Rust integration test that:
        1. Sets up a problematic position or sequence of moves known to fail
        2. Runs the engine's search or self-play for several moves
        3. Verifies that the engine handles it correctly
        4. Can be added to engine/tests/integration_tests.rs or similar
        5. Focuses on realistic game scenarios (not artificial positions)
        
        RESPONSE (JSON):
        {{
            "test_type": "integration",
            "test_name": "test_name_here",
            "title": "Short test name",
            "description": "What the integration test checks",
            "initial_fen": "Starting position (full or relative)",
            "test_code": "Complete Rust integration test function code",
            "explanation": "Why this test matters for engine stability",
            "module": "engine",
            "expected_duration_ms": "100-5000",
            "confidence": "high/medium/low"
        }}
        """)
        
        if not self.client:
            return self._placeholder_integration_test(test_focus, issue_description)
        
        try:
            response = self.client.chat.completions.create(
                model=self.model,
                messages=[{"role": "user", "content": prompt}],
                max_completion_tokens=2000,
            )
            
            response_text = response.choices[0].message.content
            
            try:
                import re
                json_match = re.search(r'\{.*\}', response_text, re.DOTALL)
                if json_match:
                    test_proposal = json.loads(json_match.group())
                else:
                    test_proposal = json.loads(response_text)
            except json.JSONDecodeError:
                test_proposal = self._placeholder_integration_test(test_focus, issue_description)
            
            return test_proposal
            
        except Exception as e:
            print(f"[candidate_generator] LLM Error generating integration test: {e}")
            return self._placeholder_integration_test(test_focus, issue_description)
    
    def _placeholder_unit_test(self, test_focus: str, issue_description: str) -> Dict:
        """Return a placeholder unit test for isolated move generation or evaluation bugs."""
        if "illegal" in test_focus:
            return {
                "test_type": "unit",
                "test_name": "test_all_moves_are_legal",
                "title": "Illegal Move Detection",
                "description": "Verify that all generated moves are legal",
                "chess_fen": "startpos",
                "test_code": dedent("""
                #[test]
                fn test_all_moves_are_legal() {
                    use crate::position::Position;
                    use crate::movegen::api::generate_pseudo_moves;
                    
                    let pos = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
                    let moves = generate_pseudo_moves(&pos);
                    
                    // Verify all generated moves are legal
                    for mv in &moves {
                        let mut out_pos = Position::default();
                        pos.apply_move_into(mv, &mut out_pos);
                        
                        // Check if our king is still safe after move
                        assert!(!out_pos.is_in_check(), "Illegal move left king in check: {:?}", mv);
                    }
                }
                """),
                "explanation": "This unit test validates move generation - no generated move should leave king in check.",
                "module": "bitboard",
                "confidence": "high"
            }
        elif "evaluation" in test_focus or "bad" in test_focus:
            return {
                "test_type": "unit",
                "test_name": "test_evaluation_differences",
                "title": "Evaluation Correctness Check",
                "description": "Test evaluation on known positions with material imbalance",
                "chess_fen": "k7/8/8/8/8/8/8/K6R w - - 0 1",
                "test_code": dedent("""
                #[test]
                fn test_evaluation_differences() {
                    use crate::position::Position;
                    
                    // Balanced starting position
                    let balanced = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
                    let balanced_eval = evaluate(&balanced);
                    
                    // White up a rook - should be significantly better for white
                    let white_up_rook = Position::from_fen("k7/8/8/8/8/8/8/K6R w - - 0 1").unwrap();
                    let white_eval = evaluate(&white_up_rook);
                    
                    // Difference should reflect rook value (roughly 500cp)
                    let diff = white_eval - balanced_eval;
                    assert!(diff > 300, "Rook advantage not reflected in eval. Diff: {}", diff);
                }
                """),
                "explanation": "Unit test that evaluation correctly rates material differences.",
                "module": "engine",
                "confidence": "high"
            }
        else:
            return {
                "test_type": "unit",
                "test_name": "test_position_apply_safety",
                "title": "Position Mutation Safety",
                "description": "Verify position application doesn't crash on standard moves",
                "chess_fen": "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                "test_code": dedent("""
                #[test]
                fn test_position_apply_safety() {
                    use crate::position::Position;
                    use crate::movegen::api::generate_pseudo_moves;
                    
                    let pos = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
                    let moves = generate_pseudo_moves(&pos);
                    
                    // Try to apply several moves without crashing
                    for (i, mv) in moves.iter().take(5).enumerate() {
                        let mut next_pos = Position::default();
                        pos.apply_move_into(mv, &mut next_pos);
                        
                        // Verify position is still valid
                        assert_eq!(next_pos.active_color(), !pos.active_color(), "Color didn't flip at move {}", i);
                    }
                }
                """),
                "explanation": "Unit test that basic position operations don't cause crashes or panics.",
                "module": "bitboard",
                "confidence": "medium"
            }

    def _placeholder_integration_test(self, test_focus: str, issue_description: str) -> Dict:
        """Return a placeholder integration test for game-level issues."""
        if "bad" in test_focus or "evaluation" in test_focus:
            return {
                "test_type": "integration",
                "test_name": "test_game_search_quality",
                "title": "Search Quality in Game",
                "description": "Verify engine doesn't lose quickly due to poor evaluation",
                "initial_fen": "startpos",
                "test_code": dedent("""
                #[test]
                fn test_game_search_quality() {
                    use crate::position::Position;
                    use crate::engine::SearchEngine;
                    
                    let mut pos = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
                    
                    // Play a short game - both sides using engine's search
                    for move_num in 0..15 {
                        let engine = SearchEngine::new();
                        let (best_move, _) = engine.search(&pos, 2);  // 2 depth for speed
                        
                        match best_move {
                            Some(mv) => {
                                let mut next_pos = Position::default();
                                pos.apply_move_into(&mv, &mut next_pos);
                                pos = next_pos;
                            },
                            None => panic!("Engine failed to find move at move {}", move_num)
                        }
                        
                        // By move 10, should still have pieces (not lost material quickly)
                        if move_num >= 10 {
                            let piece_count = pos.occupied().count_ones() as i32;
                            assert!(piece_count >= 20, "Engine lost too many pieces by move {} (only {} left)", move_num, piece_count);
                        }
                    }
                }
                """),
                "explanation": "Integration test verifying search doesn't make game-throwing moves over several moves.",
                "module": "engine",
                "expected_duration_ms": "3000-5000",
                "confidence": "medium"
            }
        else:
            return {
                "test_type": "integration",
                "test_name": "test_self_play_stability",
                "title": "Self-Play Stability",
                "description": "Run a short self-play game and verify no crashes",
                "initial_fen": "startpos",
                "test_code": dedent("""
                #[test]
                fn test_self_play_stability() {
                    use crate::position::Position;
                    use crate::movegen::api::generate_pseudo_moves;
                    
                    let mut pos = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
                    
                    for move_num in 0..20 {
                        let moves = generate_pseudo_moves(&pos);
                        if moves.is_empty() { break; }
                        
                        // Play first move
                        let mut next_pos = Position::default();
                        pos.apply_move_into(&moves[0], &mut next_pos);
                        pos = next_pos;
                    }
                    
                    // If we get here without panicking, engine is stable
                    assert!(true);
                }
                """),
                "explanation": "Integration test that engine can play moves sequentially without crashing.",
                "module": "engine",
                "expected_duration_ms": "500-2000",
                "confidence": "medium"
            }
    
    def _placeholder_improvement(self) -> Dict:
        """Return a safe, implementable improvement for testing."""
        return {
            "improvement_type": "move_ordering",
            "title": "Better Capture Ordering - MVV-LVA",
            "reasoning": (
                "Captures are searched first but currently in arbitrary order. "
                "Sorting captures by Most Valuable Victim / Least Valuable Attacker (MVV-LVA) "
                "improves move ordering efficiency and reduces nodes evaluated."
            ),
            "description": (
                "Implement MVV-LVA ordering for captures: sort by "
                "victim_value - attacker_value. Victims worth more (Queen > Rook > Bishop > Knight > Pawn) "
                "should be captured first. Light attackers (Pawns > Knights) should be preferred for captures."
            ),
            "implementation_approach": (
                "In movegen or search phase, after generating captures, sort them by:"
                "score = (victim_material_value - attacker_value) * 10 + tie_breaker. "
                "This improves alpha-beta pruning efficiency by finding better cutoffs sooner."
            ),
            "expected_elo_gain": "10-20 ELO",
            "risk_level": "low",
            "files_affected": [
                "engine/src/search/core.rs (if search handles move ordering)",
                "bitboard/src/movegen/api.rs (if ordering happens in movegen)"
            ],
            "confidence": "high"
        }


def main():
    import argparse
    
    parser = argparse.ArgumentParser(description="Generate ELO improvement proposals")
    parser.add_argument("--repo-path", required=True, help="Path to Cody repository")
    parser.add_argument("--model", default="o3-mini", help="LLM model to use")
    parser.add_argument("--api-key", default=None, help="OpenAI API key (default: env var)")
    parser.add_argument("--json", action="store_true", help="Output as JSON only")
    
    args = parser.parse_args()
    
    generator = CandidateGenerator(args.repo_path, model=args.model, api_key=args.api_key)
    proposal = generator.generate_improvement_proposal()
    
    if args.json:
        print(json.dumps(proposal, indent=2))
    else:
        print("=" * 80)
        print(f"Improvement Proposal: {proposal.get('title', 'Unknown')}")
        print("=" * 80)
        print(f"Type:       {proposal.get('improvement_type', 'N/A')}")
        print(f"Confidence: {proposal.get('confidence', 'N/A')}")
        print(f"Risk:       {proposal.get('risk_level', 'N/A')}")
        print(f"ELO Est:    {proposal.get('expected_elo_gain', 'N/A')}")
        print()
        print("Reasoning:")
        print(proposal.get('reasoning', 'N/A'))
        print()
        print("Description:")
        print(proposal.get('description', 'N/A'))
        print()
        print("Implementation:")
        print(proposal.get('implementation_approach', 'N/A'))
        print()
        print("Files to modify:")
        for f in proposal.get('files_affected', []):
            print(f"  - {f}")
        print()
        print(json.dumps(proposal, indent=2))


if __name__ == "__main__":
    sys.exit(main() or 0)
