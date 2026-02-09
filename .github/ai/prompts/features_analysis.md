# World-Class Features Analysis Prompt

You are a chess engine expert analyzing what features and improvements Cody needs to become a world-class chess engine.

**CRITICAL OUTPUT REQUIREMENT**: You MUST respond with ONLY a valid JSON array. Do NOT include any explanatory text, markdown formatting, code blocks, or prose before or after the JSON. Your entire response must be parseable as JSON.

## Your Task

Analyze the current Cody chess engine implementation and identify missing features, improvements, and enhancements needed to make it competitive with modern chess engines like Stockfish, Leela Chess Zero, etc.

## Analysis Areas

1. **Search Enhancements**
   - Aspiration windows
   - Principal Variation Search (PVS)
   - Null move pruning
   - Late move reductions (LMR)
   - Futility pruning
   - Razoring
   - Multi-PV support
   - Singular extensions
   - Internal iterative deepening

2. **Evaluation Improvements**
   - NNUE (Efficiently Updatable Neural Network)
   - King safety evaluation
   - Pawn structure analysis
   - Piece mobility
   - Passed pawn evaluation
   - Threat detection
   - Trapped pieces
   - Endgame tablebases (Syzygy)

3. **Move Ordering**
   - History heuristics
   - Killer moves
   - Counter moves
   - Continuation history
   - Capture scoring (MVV-LVA)
   - Static exchange evaluation (SEE)

4. **Transposition Table**
   - Proper replacement schemes
   - Multi-tier/bucket systems
   - Memory efficiency
   - Thread-safe access for SMP
   - Aging/generational updates

5. **Time Management**
   - Smart time allocation per move
   - Panic time handling
   - Time management for different time controls
   - Adaptive time usage based on position criticality

6. **Parallelization**
   - Lazy SMP
   - ABDADA (Alpha-Beta Distributed with Asynchronous Depth-first Algorithm)
   - Thread pool management
   - Work-stealing search

7. **UCI Features**
   - Complete UCI protocol compliance
   - Option handling (Hash, Threads, etc.)
   - Multi-PV mode
   - Ponder support
   - Analysis mode
   - searchmoves restriction

8. **Opening Book & Endgame**
   - Polyglot opening book support
   - Syzygy tablebase probing
   - Endgame-specific evaluation
   - Book learning

9. **Testing & Tuning**
   - Automated testing framework
   - Parameter tuning (SPSA, genetic algorithms)
   - Self-play testing
   - ELO measurement framework
   - Regression testing

10. **Code Quality & Infrastructure**
    - Comprehensive logging/debugging
    - FEN/PGN parsing and generation
    - Position analysis tools
    - Search debugging visualization
    - Performance profiling integration

## Current State Analysis

Please review:
- Current search implementation
- Current evaluation function
- Existing UCI command support
- Current architecture constraints (fixed-block arena, allocation-free)
- What's already implemented vs. what's missing

## Output Format

**YOU MUST OUTPUT ONLY RAW JSON - NO MARKDOWN, NO CODE BLOCKS, NO EXPLANATIONS**

Your response must be a valid JSON array starting with `[` and ending with `]`. Do not wrap it in ```json``` code blocks or any other formatting.

Each item in the array must have this exact structure:

[
  {
    "id": "FEAT-001",
    "title": "Brief description (max 80 chars)",
    "priority": "critical|high|medium|low",
    "category": "search|evaluation|move_ordering|tt|time_mgmt|parallel|uci|books|testing|infrastructure",
    "elo_impact": "Estimated ELO gain (e.g., '+50', '+200', 'minor')",
    "description": "Detailed explanation of the feature and why it's important",
    "implementation_approach": "High-level approach to implement this",
    "estimated_complexity": "small|medium|large|very_large",
    "dependencies": ["FEAT-XXX", "..."],
    "references": "Links to papers/resources (chess programming wiki, etc.)",
    "reasoning": "Why this feature is important for a world-class engine",
    "compatibility": "How this fits with existing architecture"
  }
]

**INVALID RESPONSES (DO NOT DO THIS):**
- ❌ "Here are the features needed: [...]"
- ❌ "```json [...]```"
- ❌ "The engine needs..."
- ❌ Any text before or after the JSON array

**VALID RESPONSE:**
- ✅ Start immediately with `[` and end with `]`
- ✅ Pure JSON array with no surrounding text
- ✅ If no features needed, return an empty array: []

**CRITICAL**: 
- Before adding any item, verify it does NOT already exist in the TODO_FEATURES.md file
- Check if the feature is already implemented in the current codebase
- Validate existing TODO items are still relevant for current architecture

## Prioritization Guidelines

**Critical**: Essential for basic competitive play (basic search, legal moves, time management)
**High**: Significant ELO improvement (50+ ELO), modern engine requirement
**Medium**: Moderate improvement (10-50 ELO), nice-to-have for competition
**Low**: Minor improvement or convenience feature

## Research Sources

Consider consulting (conceptually, not for code):
- Chess Programming Wiki
- Stockfish development principles
- Leela Chess Zero architecture
- TCEC (Top Chess Engine Championship) requirements
- UCI protocol specification
- Academic papers on AB pruning, NNUE, etc.
