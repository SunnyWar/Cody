# Cody Chess Engine Optimization for AMD Ryzen 7 6800H

## Hardware Profile Summary
- **Cores:** 8 cores / 16 threads (full SMT enabled)
- **Architecture:** Zen 3+ (Rembrandt, 6nm)
- **Cache:** 16 MB L3 (shared)
- **Memory:** DDR5 capable (high bandwidth)
- **ISA:** AVX2, FMA3, BMI2/PEXT (fully supported)
- **Strengths:** Parallelism, memory bandwidth, efficient execution

---

## Recommended Optimization List (Prioritized by Impact)

### Priority 1: Thread Configuration (Easy, High Impact)

**Current State:** Engine defaults to `num_threads: 1`

**Changes:**
1. **In [engine/src/api/uciapi.rs](engine/src/api/uciapi.rs)** - Set default to 8 threads:
   ```rust
   engine.set_num_threads(8);  // Was: 1
   ```

2. **In UCI option initialization** - Increase hash size for 8-threaded search:
   ```rust
   engine.set_hash_size_mb(256);  // 256 MB hash for SMT-friendly parallel access
   ```

**Why:** Your CPU has 16 threads but current engine isn't using it. With 8 threads:
- Multi-threaded search ("ABDADA" / root parallelism) gives ~6-7x speedup
- 256 MB hash is aggressive but stays near L3 boundaries
- Rayon thread pool is already set up; this just activates it

**Expected Gain:** 6-7x overall speedup

---

### Priority 2: Hash Table Tuning (Easy, Medium Impact)

**Current State:** Default hash size is 2^20 entries (~24 MB)

**Changes:**
1. **Consider three configurations based on available system memory:**
   - **Conservative:** 128 MB (1M nodes × ~128 bytes per entry)
   - **Balanced:** 256 MB (aggressive hash hits, fits well with 16 MB L3)
   - **Aggressive:** 512 MB (if system has >8 GB free)

2. **Benchmark each via UCI:**
   ```
   setoption name Hash value 128
   go depth 18 perft
   ```

**Why:** 
- 16 MB L3 cache means medium hash = cache-friendly (less memory stall)
- More hash entries reduce transposition table collisions
- Each 8 thread searches same hash concurrently → bigger table helps

**Expected Gain:** 2-4% depth improvement per doubling of hash (up to 512 MB)

---

### Priority 3: AVX2 / FMA3 Optimization (Medium Effort, Variable Impact)

**Current State:** Compiler targets generic x86_64; Ryzen 7 6800H supports AVX2 + FMA3

**Changes:**
1. **Enable target feature in [engine/Cargo.toml](engine/Cargo.toml):**
   ```toml
   [profile.release]
   rustflags = ["-C", "target-cpu=native", "-C", "target-feature=+avx2,+fma"]
   ```
   
   Or alternatively in [Cargo.toml root](Cargo.toml):
   ```toml
   [profile.release]
   rustflags = ["-C", "target-cpu=znver3"]  # Native tuning for Zen 3
   ```

2. **Apply to evaluation function** - if using float operations anywhere, FMA3 helps:
   ```rust
   // Evaluator.rs - ensure floating-point scores use FMA when possible
   // Already optimized, but adding explicit -C target-cpu ensures compiler uses it
   ```

**Why:**
- Zen 3 has dual AVX2 execution units
- FMA3 (Fused Multiply-Add) reduces instruction count in score calculations
- `target-cpu=znver3` tells LLVM: "optimize for Zen 3, not generic x86"
- Often 1-3% free improvement

**Expected Gain:** 1-3% NPS improvement

---

### Priority 4: Move Generation & Cache Efficiency (Medium Effort, High Impact)

**Current State:** Move generation uses magic bitboards (already fast)

**Changes:**
1. **Reduce intermediate allocations in move ordering** - Check [engine/src/search/core.rs](engine/src/search/core.rs):
   ```rust
   // Ensure order_moves_with_heuristics_fast doesn't clone or reallocate
   // Profile: `cargo flamegraph --release -- bench` to verify
   ```

2. **Prefetch next moves in principal variation** (if not already done):
   ```rust
   // In search loop: prefetch move ordering hints before deep recurse
   // This helps CPU branch prediction
   ```

3. **Align hot move lists to 64-byte cache lines:**
   ```rust
   // struct MoveList should have #[repr(align(64))] if not present
   ```

**Why:**
- L1 cache (32 KB) holds ~500 moves
- Avoiding pipeline stalls from memory loads → 2-3% improvement
- Zen 3 benefits from cache-line alignment

**Expected Gain:** 2-5% depending on current cache misses

---

### Priority 5: Quiescence Search Pruning (Implemented, Verify)

**Current State:** SEE-based pruning should already exist

**Verify in [engine/src/search/quiescence.rs](engine/src/search/quiescence.rs):**
```rust
const SEE_QUIET_THRESHOLD: i32 = -50;   // Allow minor losses in shallow search
const SEE_DEEP_THRESHOLD: i32 = 0;      // Only equal trades in deep search
```

**Why:** Captures queen-taking defended piece → massive q-search explosion. SEE prunes bad captures early.

**Expected Gain:** 3-5x faster on complex positions (already implemented)

---

### Priority 6: Aspiration Windows Tuning (Easy, Depends on Evaluation)

**Current State:** Aspiration window set to 25 cp, max 4 researches

**Consider tuning in [engine/src/search/search.rs](engine/src/search/search.rs):**
```rust
const ASPIRATION_START_DELTA_CP: i32 = 25;      // Narrow slightly (20)?
const ASPIRATION_MAX_RESEARCHES: usize = 4;     // Or limit to 3 to save time
const ASPIRATION_MIN_DEPTH: usize = 3;          // Start aspiration earlier (depth 2)?
```

**Why:** Aspiration windows reduce searches if evaluation is stable. Your Ryzen 7's fast clocks benefit from fewer researches.

**Expected Gain:** 1-2% depending on position volatility

---

### Priority 7: Arena Sizing (Easy, Small Gain)

**Current State:** Default arena capacity allocated in [engine/src/search/search.rs](engine/src/search/search.rs)

**Recommended tuning:**
```rust
// In Engine::new() or CodyApi initialization
let mut engine = Engine::new(16_000_000, ...);  // 16M nodes pre-allocated
engine.set_hash_size_mb(256);
engine.set_num_threads(8);
```

**Why:**
- 8 threads × 2M nodes each = 16M total arena
- Allocate once, reuse across all depths
- Reduces arena allocation overhead

**Expected Gain:** 1% (negligible but safety improvement)

---

### Priority 8: Lazy SMP Configuration (Advanced, High Impact if implemented)

**Current State:** Uses root parallelism (ABDADA)

**Future optimization** if considering Lazy SMP:
- Distribute TT shared, search trees independent
- Better scaling on 8+ cores
- More complex but can reach near-linear speedup

**For now:** Root parallelism should reach ~6.5x on 8 threads.

---

## Implementation Checklist

Task tracking for this checklist has been consolidated into `TODO.md` at the repository root.

---

## Compilation Command

```bash
# Build optimized for Ryzen 7 6800H
cargo build --release

# Or with explicit Zen 3 tuning
RUSTFLAGS="-C target-cpu=znver3 -C target-feature=+avx2,+fma3" cargo build --release

# Benchmark
cargo bench --release -p engine
```

---

## Expected Performance Summary

| Configuration | Speedup vs 1-thread Default |
|---------------|-----|
| 1 thread, 20 MB hash (current) | 1.0x (baseline) |
| 8 threads, 20 MB hash | 6.5x |
| 8 threads, 256 MB hash | 6.8x |
| 8 threads, 256 MB, AVX2 tuning | 7.0x |
| 8 threads, 256 MB, AVX2, optimized movegen | 7.5x |
| **Full optimization suite** | **8-10x** |

---

## Hardware-Specific Notes

**Ryzen 7 6800H Strengths to Exploit:**
- ✅ 2 preferred cores (#1, #5) - OS scheduler favors these; Rayon will auto-distribute  
- ✅ DDR5 memory - fast RAM bandwidth benefits Lazy SMP if ever implemented
- ✅ Dual AVX2 execution units - compiler flag enables full utilization
- ✅ Efficient Zen 3+ cores - good IPC at moderate clocks

**Potential Limitations:**
- ⚠️ 35W TDP - power limit may affect boost clock under load
- ⚠️ 16 threads on 8 cores - SMT helps throughput but adds contention on small queues

---

## Next Steps

1. **Start with Priority 1-3** (threading, hash, compiler flags) → measure with `cargo bench`
2. **Profile the result** with `flamegraph` to identify remaining bottlenecks
3. **Apply Priority 4-6** as needed based on profile results
4. **Test correctness** with `cargo test` after each change
5. **Validate performance** on a gauntlet of test positions

