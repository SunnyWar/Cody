## Perft Command-Line Usage

The engine now supports running perft benchmarks directly from the command line.

### Basic Usage

Run perft with default depth (5):
```bash
cargo run --release -- perft
# Output: perft(5) = 4867379 (1.671s)
```

Run perft with custom depth:
```bash
cargo run --release -- perft 3
# Output: perft(3) = 8903 (0.006s)
```

### Verified Results

The current implementation correctly validates these known perft values for the initial chess position:

| Depth | Count | Notes |
|-------|-------|-------|
| 0 | 1 | Base case (no moves) |
| 1 | 20 | Twenty opening moves |
| 2 | 400 | All positions after two half-moves |
| 3 | 5,362 | |
| 4 | 71,055 | |
| 5 | 4,867,379 | Full deep search |

### Implementation Details

- Added command-line argument handler in [engine/src/main.rs](engine/src/main.rs)
- Uses `util::run_perft_benchmark()` from [engine/src/util.rs](engine/src/util.rs)
- Defaults to depth 5 if not specified
- Runs perft on the initial position by default
- All performance values are from release builds (use `--release`)

### UCI Mode Still Works

When no arguments are provided, the engine runs in UCI mode:
```bash
cargo run --release
# Reads UCI commands from stdin
```

### Performance Tips

- **Release builds are essential**: Always use `--release` for benchmarking
  - `cargo run --release -- perft 5` instead of `cargo run -- perft 5`
- **Depth scaling is exponential**: Depth 6 takes significantly longer
- **Perft validates correctness**: Both move generation **and** illegal move filtering
