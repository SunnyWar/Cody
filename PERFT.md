# Perft (Performance Test) for Cody Chess Engine

## Overview

Perft is a performance testing tool for chess engines that counts the number of leaf nodes at a given depth from a position. It's used to verify move generation correctness and measure performance.

## Usage

### From Rust Code

The perft module is exported from `bitboard` and available in your code:

```rust
use bitboard::{position::Position, perft, perft_divide};

// Create a position from FEN
let pos = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

// Count leaf nodes at depth 3
let count = perft(&pos, 3);
println!("perft(3) = {}", count);

// Divide: show nodes for each move
let divide_output = perft_divide(&pos, 2);
println!("{}", divide_output);
```

### Using Engine Utilities

The `engine` crate provides convenience functions with timing:

```rust
use engine::util::{run_perft_benchmark, run_perft_divide};
use bitboard::position::Position;

let pos = Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 10");

// Run with timing
run_perft_benchmark(&pos, 4);
// Output: perft(4) = 4085603 (0.123s)

// Divide with timing
run_perft_divide(&pos, 2);
```

## Examples

### Initial Position
```
perft(1) = 20  (20 legal moves)
perft(2) = 400
perft(3) = 5,362
```

### Kiwipete (famous complex position)
```
FEN: r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 10

perft(1) = 48   (48 legal moves)
perft(2) = 2,039
perft(3) = 97,862
```

## Running Tests

```bash
# Run perft unit tests
cargo test --lib perft

# Run all tests
cargo test
```

## How Perft Works

Perft recursively counts legal move sequences:

- **Depth 0:** Always returns 1 (no moves to make)
- **Depth 1:** Returns the count of legal moves from the position
- **Depth > 1:** For each legal move, applies it and recursively counts perft(depth-1)

The algorithm verifies:
1. **Move generation** is complete and correct
2. **Legal move filtering** (no illegal moves)
3. **Move application** (position updates correctly)
4. **Game rules** (castling, en passant, captures, etc.)

## Performance Tips

- Use **release builds** for accurate performance measurement: `cargo build --release`
- Perft is CPU-bound; measure wall-clock time
- Larger depths get exponentially slower; depth 5 may take seconds
- Early depths (1-3) are useful for quick validation

## Debugging Move Generation

Use `perft_divide(pos, depth)` to see which moves contribute how many leaf nodes. This helps identify where move generation might be incorrect.
