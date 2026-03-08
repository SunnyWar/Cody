# AVX2 SIMD Implementation

## Overview

Implemented comprehensive AVX2/SIMD optimizations for Cody chess engine targeting AMD Ryzen 7 6800H (Zen 3+).

## Performance Impact

- **Baseline**: 1.34M NPS (nodes per second) at depth 13
- **With AVX2**: 1.50M NPS at depth 13
- **Improvement**: ~12% performance increase

## Implementation Details

### 1. SIMD Data Types (bitboard/src/intrinsics.rs)

#### SimdU64x4
- 256-bit AVX2 vector for 4 x u64 values
- Used for parallel bitboard operations
- **Operations**:
  - `popcnt_parallel()`: Count bits in 4 bitboards simultaneously
  - `and()`, `or()`, `xor()`: Parallel bitwise operations
  - `not()`: Parallel bitwise negation
  - `any_nonzero()`, `all_zero()`: Zero testing

#### SimdI32x8
- 256-bit AVX2 vector for 8 x i32 values
- Used for parallel evaluation scoring
- **Operations**:
  - `add()`, `sub()`: Parallel arithmetic
  - `horizontal_sum()`: Reduce 8 values to single sum
  - `max()`, `min()`: Parallel min/max
  - `splat()`: Broadcast single value to all lanes

### 2. Evaluation Optimizations (engine/src/search/evaluator.rs)

#### compute_phase() - SIMD Parallel Popcount
**Before**: 12 sequential popcnt operations (6 piece types × 2 colors)
**After**: Batch 4 bitboards per side using `SimdU64x4::popcnt_parallel()`

```rust
// Load 4 piece bitboards into SIMD vector
let vec = SimdU64x4::new(pawns, knights, bishops, rooks);
let counts = vec.popcnt_parallel(); // Parallel popcount
```

**Impact**: Reduces function overhead by processing multiple bitboards in parallel.

#### evaluate_bishop_pair() - Parallel Bishop Counting
**Before**: 2 sequential popcnt operations
**After**: Single SIMD parallel popcount for both colors

```rust
let vec = SimdU64x4::new(white_bishops_bb, black_bishops_bb, 0, 0);
let counts = vec.popcnt_parallel();
```

#### evaluate_pieces_batch_simd() - Batch PST Evaluation
New function for SIMD-optimized piece-square table evaluation.

**When**: Processes ≥8 pieces (pawns typically have 8)
**How**: 
1. Load 8 PST values (midgame + endgame) into SIMD vectors
2. Perform phase-blended evaluation in parallel
3. Horizontal sum for final score

```rust
// Process 8 pawns simultaneously
let mid_vec = SimdI32x8::new(
    mid_table[indices[0]], mid_table[indices[1]], ..., mid_table[indices[7]]
);
let end_vec = SimdI32x8::new(
    end_table[indices[0]], end_table[indices[1]], ..., end_table[indices[7]]
);
// Blend in parallel...
let total_score = blended.horizontal_sum();
```

**Impact**: Most effective for pawn evaluation where 8+ pieces are common.

### 3. Architecture Support

#### Target Platform
- **CPU**: AMD Ryzen 7 6800H (Zen 3+, 6nm)
- **Features**: AVX2, FMA3, BMI2, POPCNT
- **Cache**: 64-byte L1 cache lines

#### Compiler Configuration (.cargo/config.toml)
```toml
[build]
rustflags = [
    "-C", "target-cpu=znver3",
    "-C", "target-feature=+avx2,+fma,+bmi2"
]
```

#### Fallback Paths
All SIMD operations include scalar fallbacks for non-AVX2 targets:

```rust
#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
unsafe {
    // AVX2 intrinsics path
}

#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2")))]
{
    // Portable scalar fallback
}
```

## Testing

### Unit Tests
- **14 intrinsics tests**: All passing
- **177 total tests**: All passing (bitboard + engine + integration)

### Test Coverage
```
test intrinsics::tests::test_simd_u64x4_popcnt ... ok
test intrinsics::tests::test_simd_u64x4_bitwise ... ok
test intrinsics::tests::test_simd_u64x4_zero_checks ... ok
test intrinsics::tests::test_simd_i32x8_arithmetic ... ok
test intrinsics::tests::test_simd_i32x8_minmax ... ok
test intrinsics::tests::test_simd_i32x8_splat ... ok
```

### Engine Validation
- **Depth 13** (startpos): 1.50M NPS in 933ms
- **Depth 10** (Kiwipete): 1.16M NPS in 2167ms
- **Result**: Correct bestmove, no regressions

## Code Organization

### Intrinsics Module (bitboard/src/intrinsics.rs)
- **Lines**: ~850 (including tests)
- **Sections**:
  1. Prefetch operations (L1/L2 cache hints)
  2. Bit manipulation (POPCNT, TZCNT, LZCNT, BLSR, BLSI, PEXT, PDEP)
  3. SIMD types (SimdU64x4, SimdI32x8)
  4. Comprehensive tests

### Integration Points
- **bitboard/src/movelist.rs**: Prefetch in move iteration
- **bitboard/src/bitboardmask.rs**: Intrinsics in iterators
- **bitboard/src/attack.rs**: King square extraction
- **bitboard/src/bitboard.rs**: PEXT for magic bitboards
- **engine/src/search/core.rs**: Prefetch in search loop
- **engine/src/search/evaluator.rs**: SIMD evaluation

## Future Optimization Opportunities

### 1. Move Generation
- Parallel attack generation for multiple pieces
- SIMD ray-cast for bishops/rooks/queens

### 2. Move Ordering
- Batch score 8 moves simultaneously using `SimdI32x8`
- Parallel MVV-LVA calculation

### 3. Attack Detection
- Parallel "is_square_attacked" checks
- Batch king safety evaluation

### 4. Transposition Table
- SIMD hash computation
- Parallel probe for multiple positions

### 5. Search
- Vectorized aspiration window bounds
- Parallel fail-high detection

## Best Practices

### When to Use SIMD
✅ **Good candidates**:
- Batch operations on arrays (≥8 elements)
- Parallel bitboard operations
- Horizontal reductions (sums, min/max)
- High-frequency hot paths

❌ **Poor candidates**:
- Single operations
- Data-dependent control flow
- Small arrays (<4 elements)
- Cold code paths

### Performance Tips
1. **Alignment**: Use `#[repr(align(32))]` for SIMD types
2. **Batch size**: Process 8 elements at a time for `_mm256` operations
3. **Fallbacks**: Always provide scalar fallback for portability
4. **Testing**: Verify both AVX2 and fallback paths work correctly
5. **Profiling**: Measure actual performance impact (SIMD isn't always faster)

## References

- **Intel Intrinsics Guide**: https://www.intel.com/content/www/us/en/docs/intrinsics-guide/
- **Rust std::arch**: https://doc.rust-lang.org/stable/core/arch/
- **AMD Zen 3 Optimization Guide**: https://www.amd.com/content/dam/amd/en/documents/processor-tech-docs/software-optimization-guides/56665.zip

## Commit Summary

**Added**:
- `SimdU64x4` and `SimdI32x8` types in intrinsics module
- SIMD-optimized `compute_phase()` function
- SIMD-optimized `evaluate_bishop_pair()` function
- `evaluate_pieces_batch_simd()` for batch PST evaluation
- 6 new SIMD unit tests

**Modified**:
- Evaluator to use SIMD for pawn evaluation
- Test suite expanded to 177 tests

**Performance**: +12% NPS improvement (1.34M → 1.50M NPS @ depth 13)
