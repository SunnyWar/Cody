"""
Performance Agent - Orchestrates performance optimization for the chess engine.

This agent focuses on targeted optimizations in critical paths:
1. Single-function optimization: Optimize one random function from a perf-critical file
2. File-level recommendations: Analyze a file and pick one recommendation to implement
3. Bitboard operation optimization: Improve bitboard operations as they're in the hot path
4. Memory access patterns: Optimize cache locality and memory patterns
5. Hotspot reduction: Target most-called functions from profiling data
6. Integer/shift operations: Optimize bit manipulation and arithmetic operations
7. Inlining opportunities: Mark hot functions for inlining
8. Allocation-free improvements: Remove heap allocations from hot paths

Each iteration applies ONE strategy to avoid destabilizing the codebase.
"""

import json
import os
import random
import subprocess
from pathlib import Path
from typing import Optional, Tuple

from openai import OpenAI

from state.cody_state import CodyState

DEFAULT_MAX_PHASE_ITERATIONS = 8

PERFORMANCE_CRITICAL_FILES = [
    "bitboard/src/movegen.rs",      # Move generation (called per position)
    "bitboard/src/position.rs",     # Position apply_move (core hot path)
    "bitboard/src/attack.rs",       # Bitboard attacks (used in move legality)
    "bitboard/src/piecebitboards.rs",  # Piece bitboard operations
    "engine/src/search/engine.rs",  # Main search loop
    "engine/src/core/arena.rs",     # Arena allocation (hot path)
]

PERFORMANCE_STRATEGIES = [
    {
        "name": "Single-function optimization",
        "instruction": (
            "You are given a performance-critical function from the chess engine. "
            "Optimize it for maximum speed while maintaining identical behavior:\n"
            "1. Look for unnecessary clones or copies\n"
            "2. Look for redundant computations\n"
            "3. Look for inefficient field accesses\n"
            "4. Look for branch misprediction opportunities\n"
            "5. Look for cache-unfriendly memory access patterns\n"
            "If optimization is possible, provide the change as a unified diff. "
            "If no material speedup is possible, explain briefly why."
        ),
    },
    {
        "name": "Bitboard operation optimization",
        "instruction": (
            "Bitboard operations are in the hottest path of the chess engine. "
            "Review the bitboard code to find:\n"
            "1. Unnecessary intermediate bitboards that could be computed in one expression\n"
            "2. Multiple bitboard iterations that could be combined\n"
            "3. Bitwise operations that could be reordered for efficiency\n"
            "4. Masks that could be precomputed as constants\n"
            "Provide ONE optimization change as a unified diff if found. "
            "If no clear improvement exists, explain why."
        ),
    },
    {
        "name": "File-level analysis with recommendation",
        "instruction": (
            "Analyze one performance-critical file for optimization opportunities:\n"
            "1. Identify the 3 most impactful optimization opportunities\n"
            "2. Rank them by estimated performance gain\n"
            "3. Implement the #1 ranked optimization as a unified diff\n"
            "Focus on concrete, measurable improvements (not cosmetic). "
            "If no material optimization exists, explain why the code is already well-optimized."
        ),
    },
    {
        "name": "Cache locality improvement",
        "instruction": (
            "Optimize memory access patterns for better cache locality:\n"
            "1. Look for struct fields accessed in non-contiguous order\n"
            "2. Look for sequential memory access that could be improved\n"
            "3. Look for unnecessary pointer chasing in hot loops\n"
            "4. Consider struct field reordering for better cache hits\n"
            "Provide ONE change as a unified diff that improves cache efficiency. "
            "If no improvement is viable, explain briefly."
        ),
    },
    {
        "name": "Hot path allocation reduction",
        "instruction": (
            "The Cody engine uses a fixed-block allocator and must be allocation-free in hot paths. "
            "Find and remove:\n"
            "1. Vec allocations in search loops\n"
            "2. Box allocations in move generation\n"
            "3. String allocations in performance-critical functions\n"
            "4. Any heap allocation in frequently-called functions\n"
            "Replace with stack-based or arena-allocated alternatives. "
            "Provide ONE change as a unified diff. If no heap allocations exist in hot paths, "
            "confirm the code already follows the allocation-free constraint."
        ),
    },
    {
        "name": "Branching and prediction optimization",
        "instruction": (
            "Optimize code to reduce branch prediction misses:\n"
            "1. Look for unpredictable branches in loops\n"
            "2. Look for branches that could be replaced with bitwise operations\n"
            "3. Look for conditional code that could be branchless\n"
            "4. Consider reordering conditions for better prediction patterns\n"
            "Provide ONE optimization as a unified diff. "
            "If the code already minimizes branches well, explain why no change is needed."
        ),
    },
    {
        "name": "Loop optimization and iteration",
        "instruction": (
            "Optimize loops in performance-critical code:\n"
            "1. Look for loop bounds that could be precomputed\n"
            "2. Look for redundant operations inside loops\n"
            "3. Look for opportunities to hoist invariants out of loops\n"
            "4. Look for loop fusion opportunities (combining multiple passes)\n"
            "Provide ONE loop optimization as a unified diff. "
            "If looping is already well-optimized, explain briefly why."
        ),
    },
]


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


def _select_model(config: dict, phase: str = "performance") -> str:
    """Select the appropriate model for performance optimization."""
    models = config.get("models", {}) if isinstance(config, dict) else {}
    # Try phase-specific model first, fall back to clippy, then config default
    return models.get(phase) or models.get("clippy") or config.get("model") or "gpt-4o"


def _get_system_prompt() -> str:
    """System prompt for performance optimization agent."""
    return """You are Cody's PerformanceAgent.
Goal: Optimize the Cody chess engine for maximum speed while maintaining correctness.

ARCHITECTURE CONSTRAINTS:
- Fixed-block allocator: Search nodes live in preallocated arenas (engine/src/core/arena.rs)
- Allocation-free hot path: No heap allocations in search/move generation
- Pseudo-legal moves: Move generation doesn't check legality; legality validated separately
- Bitboard-based: All position representation uses bitboards for speed

PERFORMANCE-CRITICAL FILES (in order of impact):
1. bitboard/src/movegen.rs - Move generation (called per position, millions times per search)
2. bitboard/src/position.rs - Position apply_move (core operation in search)
3. bitboard/src/attack.rs - Square attack detection (used in move legality)
4. engine/src/search/engine.rs - Main search loop
5. engine/src/core/arena.rs - Arena allocation patterns

REQUIRED FORMAT FOR CHANGES:
- Provide changes as a UNIFIED DIFF in a markdown diff block
- Format example:
    --- a/bitboard/src/movegen.rs
    +++ b/bitboard/src/movegen.rs
    @@ -157,5 +157,4 @@
         let mut result = Vec::new();
    -    for target in targets {
    +    for target in targets.0.iter_bits() {
         }
- The @@ hunk header MUST include actual line numbers: @@ -157,5 +157,4 @@
- NEVER use @@ @@ without numbers - this is INVALID
- NEVER use *** markers
- Put diff inside a markdown code block with 'diff' language tag

OPTIMIZATION GOALS:
- Target ≥5% performance improvement per change
- Maintain correctness (pass perft tests, benchmarks)
- Preserve allocation-free constraint in hot paths
- No behavior changes - only performance improvements
- Avoid code size bloat unless justified by speed gains

SAFETY RULES:
- NEVER add #[allow(...)], #[warn(...)], or suppression attributes
- NEVER change public APIs
- Each change must be independently correct and testable
- When in doubt, explain why optimization is not possible instead of making risky changes
"""


def _get_system_prompt_for_strategy(strategy_name: str) -> str:
    """Get system prompt combined with specific strategy instructions."""
    base_prompt = _get_system_prompt()
    
    # Find the strategy
    for strat in PERFORMANCE_STRATEGIES:
        if strat["name"] == strategy_name:
            return (
                base_prompt + "\n\n"
                f"OPTIMIZATION STRATEGY FOR THIS ITERATION:\n{strat['instruction']}"
            )
    
    return base_prompt


def _read_file_head_snippet(
    repo_path: str, file_path: str, max_lines: int = 100
) -> str:
    """Read first N lines of a file with line numbers."""
    full_path = file_path
    if not os.path.isabs(full_path):
        full_path = os.path.join(repo_path, file_path)
    if not os.path.exists(full_path):
        return ""
    try:
        lines = Path(full_path).read_text(encoding="utf-8").splitlines()
    except Exception:
        return ""

    shown = lines[:max_lines]
    numbered = [f"{i+1:4d} | {line}" for i, line in enumerate(shown)]
    try:
        rel_path = os.path.relpath(full_path, repo_path)
    except ValueError:
        rel_path = full_path
    return f"File: {rel_path}\n" + "\n".join(numbered)


def _select_performance_file(repo_path: str) -> str:
    """Select a random performance-critical file that exists."""
    random.shuffle(PERFORMANCE_CRITICAL_FILES)
    for file_path in PERFORMANCE_CRITICAL_FILES:
        full_path = Path(repo_path) / file_path
        if full_path.exists():
            return file_path
    # Fallback to first file if none exist (shouldn't happen)
    return PERFORMANCE_CRITICAL_FILES[0]


def _select_function_from_file(repo_path: str, file_path: str) -> Tuple[str, int, str]:
    """
    Extract a random function/struct from a Rust file.
    Returns: (function_name, start_line, function_code)
    """
    full_path = Path(repo_path) / file_path
    if not full_path.exists():
        return ("unknown", 0, "")

    try:
        content = full_path.read_text(encoding="utf-8")
    except Exception:
        return ("unknown", 0, "")

    lines = content.splitlines()
    functions = []

    # Find all function/struct definitions
    for i, line in enumerate(lines):
        stripped = line.strip()
        if (
            (stripped.startswith("pub fn ") or stripped.startswith("fn "))
            or (stripped.startswith("pub struct ") or stripped.startswith("struct "))
            or (stripped.startswith("impl "))
        ):
            # Extract name
            if "fn " in stripped:
                name_part = stripped.split("fn ", 1)[1].split("(", 1)[0].strip()
            elif "struct " in stripped:
                name_part = stripped.split("struct ", 1)[1].split("{", 1)[0].strip()
            elif "impl " in stripped:
                name_part = stripped.split("impl ", 1)[1].split("{", 1)[0].strip()
            else:
                continue

            functions.append((name_part, i + 1))  # i+1 for 1-indexed

    if not functions:
        return ("unknown", 0, "")

    # Pick a random function
    func_name, func_line = random.choice(functions)

    # Grab 30 lines of context
    start_idx = max(0, func_line - 1)
    end_idx = min(len(lines), func_line + 29)
    func_code_lines = lines[start_idx:end_idx]
    func_code = "\n".join(
        [f"{start_idx + i + 1:4d} | {line}" for i, line in enumerate(func_code_lines)]
    )

    return (func_name, func_line, func_code)


def _collect_performance_context(repo_path: str) -> Tuple[str, str]:
    """
    Collect context for performance optimization.
    Returns: (context_text, file_path_used)
    """
    # Randomly pick a performance-critical file
    perf_file = _select_performance_file(repo_path)

    # Randomly decide on strategy:
    # - 40% show the file and ask for top recommendations
    # - 60% show a single function and ask to optimize it
    strategy_choice = random.random()

    if strategy_choice < 0.4:
        # Strategy: analyze whole file
        context = f"""
PERFORMANCE ANALYSIS TASK
=========================
File: {perf_file}

Review the first 100 lines of this performance-critical file:

---
{_read_file_head_snippet(repo_path, perf_file, max_lines=100)}
---

Analyze it for optimization opportunities and implement your recommended optimization.
"""
    else:
        # Strategy: optimize single function
        func_name, func_line, func_code = _select_function_from_file(repo_path, perf_file)
        context = f"""
FUNCTION OPTIMIZATION TASK
==========================
File: {perf_file}
Function: {func_name}
Location: Line {func_line}

Review this performance-critical function and optimize it:

---
{func_code}
---

Find inefficiencies (unnecessary copies, redundant computation, cache misses, etc.)
and provide an optimized version.
"""

    return (context, perf_file)


def _run_benchmarks(repo_path: str) -> str:
    """Run benchmarks to measure performance."""
    try:
        result = subprocess.run(
            ["cargo", "bench", "-p", "engine", "--", "--quiet"],
            cwd=repo_path,
            capture_output=True,
            text=True,
            timeout=120,
        )
        return result.stdout + result.stderr
    except Exception as e:
        return f"Benchmark failed: {e}"


def performance_agent(state: CodyState) -> CodyState:
    """Main performance optimization agent."""
    from datetime import datetime

    print(f"[cody-graph] performance_agent: START", flush=True)

    api_key = os.environ.get("OPENAI_API_KEY")
    if not api_key:
        result_state = {
            **state,
            "last_command": "performance_llm_think",
            "last_output": "Missing OPENAI_API_KEY environment variable.",
            "status": "error",
        }
        print(f"[cody-graph] performance_agent: ERROR - {result_state['last_output']}", flush=True)
        print("[cody-graph] performance_agent: END (error)", flush=True)
        return result_state

    config = _load_config(state.get("repo_path", ""))
    current_iteration = int(state.get("phase_iteration", 0) or 0)
    config_max_iterations = int(config.get("max_phase_iterations", DEFAULT_MAX_PHASE_ITERATIONS))
    max_iterations = len(PERFORMANCE_STRATEGIES)

    print(
        f"[cody-graph] [DIAG] Phase iteration: {current_iteration}/{max_iterations}",
        flush=True,
    )

    if current_iteration >= max_iterations:
        done_msg = (
            "Performance phase completed: all optimization strategies have been attempted"
        )
        result_state = {
            **state,
            "last_command": "performance_phase_complete",
            "last_output": done_msg,
            "status": "ok",
            "phase_iteration": max_iterations,
        }
        print(f"[cody-graph] performance_agent: {done_msg}", flush=True)
        print("[cody-graph] performance_agent: END (phase complete)", flush=True)
        return result_state

    repo_path = state.get("repo_path", "")
    client = OpenAI(api_key=api_key)

    # Pick a strategy for this iteration
    strategy = PERFORMANCE_STRATEGIES[current_iteration]
    print(
        f"[cody-graph] [DIAG] Applying strategy: {strategy['name']}",
        flush=True,
    )

    # Collect context for this iteration
    context, perf_file_used = _collect_performance_context(repo_path)

    # Get architecture context
    arch_md_path = Path(repo_path) / "architecture.md"
    arch_context = ""
    if arch_md_path.exists():
        try:
            arch_context = arch_md_path.read_text(encoding="utf-8")[:2000]
        except Exception:
            pass

    # Get latest benchmark results (for reference)
    bench_context = ""
    bench_path = Path(repo_path) / "engine" / "benches" / "bench.rs"
    if bench_path.exists():
        try:
            bench_context = bench_path.read_text(encoding="utf-8")[:1500]
        except Exception:
            pass

    # Prepare the LLM request with system prompt as first message
    system_prompt = _get_system_prompt_for_strategy(strategy["name"])
    
    messages = [
        {
            "role": "system",
            "content": system_prompt,
        },
        {
            "role": "user",
            "content": f"""
{context}

ARCHITECTURE CONTEXT (condensed):
{arch_context if arch_context else "(architecture.md not available)"}

BENCHMARK REFERENCE:
{bench_context if bench_context else "(benches not available)"}

Provide your optimization as a UNIFIED DIFF in a markdown code block.
Focus ONLY on this one optimization. If no improvement is possible, explain briefly why.
""",
        }
    ]

    try:
        print(f"[cody-graph] [DIAG] Requesting LLM optimization...", flush=True)
        resp = client.chat.completions.create(
            model=_select_model(config, "performance"),
            messages=messages,
        )

        llm_output = resp.choices[0].message.content if resp.choices else ""
        print(f"[cody-graph] [DIAG] LLM response received ({len(llm_output)} chars)", flush=True)

        result_state = {
            **state,
            "last_command": "performance_llm_optimize",
            "last_output": llm_output,
            "status": "ok",
            "last_file": perf_file_used,
            "timestamp": datetime.now().isoformat(),
            "phase_iteration": current_iteration + 1,  # Increment for next strategy
        }

        print("[cody-graph] performance_agent: END (success)", flush=True)
        return result_state

    except Exception as e:
        error_msg = f"LLM request error: {str(e)}"
        result_state = {
            **state,
            "last_command": "performance_llm_optimize",
            "last_output": error_msg,
            "status": "error",
        }
        print(f"[cody-graph] performance_agent: ERROR - {error_msg}", flush=True)
        print("[cody-graph] performance_agent: END (error)", flush=True)
        return result_state
