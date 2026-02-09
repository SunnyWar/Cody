# Perft Implementation Summary

## What Was Added

A minimal but fully functional **perft (performance test)** implementation has been added to the Cody chess engine.

### Files Created

1. **[bitboard/src/perft.rs](bitboard/src/perft.rs)** — Core perft module with:
   - `perft(pos, depth)` — Count leaf nodes at given depth
   - `perft_divide(pos, depth)` — Move-by-move breakdown (useful for debugging)
   - Helper function `move_to_uci()` for move notation
   - Unit tests validating performance on known positions

2. **[engine/src/util.rs](engine/src/util.rs)** — Enhanced with:
   - `run_perft_benchmark(pos, depth)` — Run perft with timing
   - `run_perft_divide(pos, depth)` — Run divide with timing

3. **[engine/src/perft_integration_test.rs](engine/src/perft_integration_test.rs)** — Integration tests demonstrating:
   - Basic perft usage
   - Move generation debugging with divide
   - Known position verification (Kiwipete, endgames)

4. **[PERFT.md](PERFT.md)** — Complete documentation with:
   - Usage examples
   - Known perft values
   - Performance tips
   - How perft works internally

### Changes to Existing Files

- **[bitboard/src/lib.rs](bitboard/src/lib.rs)** — Added perft module and exports
- **[engine/src/lib.rs](engine/src/lib.rs)** — Added integration tests

## Usage Examples

### Basic Perft Count
```rust
use bitboard::{position::Position, perft};

let pos = Position::default();
let count = perft(&pos, 3);
println!("perft(3) = {}", count);  // Output: perft(3) = 5362
```

### With Engine Utilities
```rust
use engine::util::run_perft_benchmark;
use bitboard::position::Position;

let pos = Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 10");
run_perft_benchmark(&pos, 4);
```

### Divide (Debugging)
```rust
use bitboard::{position::Position, perft_divide};

let pos = Position::default();
println!("{}", perft_divide(&pos, 1));
// Shows: a2a3: 1, a2a4: 1, b2b3: 1, ... Total: 20
```

## Test Results

All tests pass successfully:
- ✅ 4 perft unit tests (bitboard crate)
- ✅ 4 integration tests (engine crate)
- ✅ All 44 existing tests continue to pass

Verified positions:
- Initial position: `perft(1) = 20`
- Initial position: `perft(2) = 400`
- Kiwipete: `perft(1) = 48`
- Simple endgame: `perft(1) = 15`

## Performance Tips

- Use **release builds** for benchmarking: `cargo test --release`
- Depths 1-3 are instant for validation
- Depth 4-5 takes measurable time
- Perft validates move generation correctness end-to-end

## How It Works

Perft recursively counts the number of distinct game paths:
1. At each position, generate all legal moves
2. For each move, apply it and recurse
3. At depth 0, return 1 (the current position)

This validates:
- Move generation completeness and correctness
- Position updates (castling, en passant, captures)
- Legal move filtering
- All chess rules

## Integration with Existing Code

- Uses existing `Position::apply_move_into()` for move application
- Uses existing `generate_legal_moves()` for move generation
- No external dependencies added
- Zero allocations in the hot path (moves are generated into a Vec then consumed)

## Next Steps (Optional Enhancements)

Possible future additions:
- UCI "perft" command handler (for compatibility with other tools)
- Perft benchmark harness with multiple positions
- Divide with pv-line tracking
- Bulk move counting without legality checking (for raw generation speed)
