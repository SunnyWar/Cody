# Threading Status and Performance

## Summary (March 2026)

**Threading now works correctly and provides 2-2.4x speedup with 2+ threads at deep searches (depth ≥12).**

## Fixed Issues

1. **Thread pool recreation** - Previously created a new thread pool on every depth iteration. Now uses a persistent pool.

2. **TT isolation** - Each thread had a tiny 2-entry TT. Now threads have 16MB thread-local TTs with proper reuse.

3. **Arena allocation** - Each thread allocated a full arena for every move. Now uses thread-local storage to reuse arenas.

4. **Shallow search overhead** - Added PARALLEL_MIN_DEPTH=12 threshold; shallow searches use fast serial path.

5. **Node explosion** - Implemented shared atomic alpha across threads to enable alpha-beta cutoffs. Node count now comparable to serial search.

## Performance Benchmarks

**Depth 10 (uses serial):**
- Any thread count: ~380ms (threshold bypasses parallel overhead)

**Depth 12 (uses parallel with shared alpha):**
- 1 thread: 1885ms, 2.7M nodes
- 2 threads: 778ms, 1.2M nodes (**2.4x speedup**)
- 4 threads: 775ms, 1.2M nodes (2.4x speedup)  
- 8 threads: 788ms, 1.2M nodes (2.4x speedup)

**Why speedup plateaus at 2 threads:**
- ~20 legal moves at root, so beyond 2-4 threads there's limited parallelism
- Shared alpha reduces parallelism (good for efficiency, limits scaling)
- Thread coordination overhead

## Recommendation

**Use Threads=2 or Threads=4** for 2-2.4x speedup at deep searches.  
Single-threaded remains fast due to automatic fallback for shallow depths.

## Technical Implementation

- `Engine` stores persistent `thread_pool: Option<rayon::ThreadPool>`
- TT is `Arc<RwLock<TranspositionTable>>` (shared for serial, thread-local for parallel)
- Thread-local arenas/TTs via `thread_local!` macro reduce allocation overhead
- Shared `AtomicI32` alpha enables cutoffs across threads
- Automatic threshold at depth 12 selects optimal mode

See [engine/src/search/search.rs](engine/src/search/search.rs) for implementation details.