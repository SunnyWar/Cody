//! Platform-specific intrinsics and SIMD operations.
//!
//! This module provides a centralized location for low-level CPU-specific
//! optimizations including prefetch hints, SIMD operations, and other
//! architecture-specific features.

/// Prefetch a memory address into L1 cache.
///
/// This is a performance hint to the CPU to load data into cache before it's
/// accessed. On unsupported targets this is a no-op.
///
/// # Arguments
/// * `addr` - Pointer to memory location to prefetch
pub fn prefetch_read<T>(addr: *const T) {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::x86_64::_mm_prefetch(addr as *const i8, core::arch::x86_64::_MM_HINT_T0);
    }

    #[cfg(all(target_arch = "x86", target_feature = "sse"))]
    unsafe {
        core::arch::x86::_mm_prefetch(addr as *const i8, core::arch::x86::_MM_HINT_T0);
    }

    // Other targets: no-op, let the optimizer do its work
    #[cfg(not(any(
        target_arch = "x86_64",
        all(target_arch = "x86", target_feature = "sse")
    )))]
    {
        let _ = addr; // Suppress unused variable warning
    }
}

/// Prefetch a memory address into L2 cache (less aggressive than L1).
///
/// Similar to `prefetch_read` but targets L2 cache instead of L1.
/// Useful for data that will be accessed soon but not immediately.
#[allow(dead_code)]
pub fn prefetch_read_l2<T>(addr: *const T) {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::x86_64::_mm_prefetch(addr as *const i8, core::arch::x86_64::_MM_HINT_T1);
    }

    #[cfg(all(target_arch = "x86", target_feature = "sse"))]
    unsafe {
        core::arch::x86::_mm_prefetch(addr as *const i8, core::arch::x86::_MM_HINT_T1);
    }

    #[cfg(not(any(
        target_arch = "x86_64",
        all(target_arch = "x86", target_feature = "sse")
    )))]
    {
        let _ = addr;
    }
}

/// Prefetch for write (exclusive cache line ownership).
///
/// Hints that the cache line will be modified soon. On x86 this may use
/// prefetchw if available, otherwise falls back to regular prefetch.
#[allow(dead_code)]
pub fn prefetch_write<T>(addr: *const T) {
    // For now, use the same as prefetch_read since _mm_prefetch doesn't
    // distinguish read vs write on most x86 CPUs (prefetchw is a separate
    // instruction but not exposed through _mm_prefetch)
    prefetch_read(addr);
}

// ============================================================================
// Bit Manipulation Intrinsics (Critical for bitboard operations)
// ============================================================================

/// Count the number of set bits (population count).
///
/// Critical for chess engines: counting pieces, mobility, control, etc.
/// Uses hardware POPCNT instruction when available, software fallback
/// otherwise.
pub fn popcnt(x: u64) -> u32 {
    #[cfg(target_arch = "x86_64")]
    {
        // x86_64 may have popcnt (SSE4.2+). Use feature detection at compile time.
        #[cfg(target_feature = "popcnt")]
        unsafe {
            return core::arch::x86_64::_popcnt64(x as i64) as u32;
        }

        #[cfg(not(target_feature = "popcnt"))]
        {
            x.count_ones()
        }
    }

    #[cfg(all(target_arch = "x86", target_feature = "popcnt"))]
    unsafe {
        core::arch::x86::_popcnt32(x as i32) as u32
            + core::arch::x86::_popcnt32((x >> 32) as i32) as u32
    }

    #[cfg(not(any(
        target_arch = "x86_64",
        all(target_arch = "x86", target_feature = "popcnt")
    )))]
    {
        x.count_ones()
    }
}

/// Count trailing zeros (find index of least significant set bit).
///
/// Used for: extracting square indices from bitboards, iterating pieces.
/// Returns 64 if x == 0.
pub fn trailing_zeros(x: u64) -> u32 {
    if x == 0 {
        return 64;
    }

    trailing_zeros_nonzero(x)
}

/// Count trailing zeros for nonzero input.
///
/// Caller must guarantee `x != 0`.
pub fn trailing_zeros_nonzero(x: u64) -> u32 {
    debug_assert!(x != 0);

    #[cfg(target_arch = "x86_64")]
    {
        #[cfg(target_feature = "bmi1")]
        unsafe {
            return core::arch::x86_64::_tzcnt_u64(x) as u32;
        }

        #[cfg(not(target_feature = "bmi1"))]
        {
            x.trailing_zeros()
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        x.trailing_zeros()
    }
}

/// Count leading zeros (find index of most significant set bit).
///
/// Used for: determining board regions, high-priority pieces.
/// Returns 64 if x == 0.
pub fn leading_zeros(x: u64) -> u32 {
    #[cfg(target_arch = "x86_64")]
    {
        #[cfg(target_feature = "lzcnt")]
        unsafe {
            return core::arch::x86_64::_lzcnt_u64(x) as u32;
        }

        #[cfg(not(target_feature = "lzcnt"))]
        {
            x.leading_zeros()
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        x.leading_zeros()
    }
}

/// Reset the least significant set bit (x & (x - 1)).
///
/// Critical for fast bitboard iteration: repeatedly extract and clear LSB.
/// Example: while bb != 0 { let sq = trailing_zeros(bb); bb = blsr(bb); ... }
pub fn blsr(x: u64) -> u64 {
    if x == 0 {
        return 0;
    }

    blsr_nonzero(x)
}

/// Reset the least significant set bit for nonzero input.
///
/// Caller must guarantee `x != 0`.
pub fn blsr_nonzero(x: u64) -> u64 {
    debug_assert!(x != 0);

    #[cfg(target_arch = "x86_64")]
    {
        #[cfg(target_feature = "bmi1")]
        unsafe {
            return core::arch::x86_64::_blsr_u64(x);
        }

        #[cfg(not(target_feature = "bmi1"))]
        {
            x & (x - 1)
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        x & (x - 1)
    }
}

/// Extract the least significant set bit (x & -x).
///
/// Isolates a single bit for masking operations.
/// Example: get the lowest piece on a bitboard without removing it.
pub fn blsi(x: u64) -> u64 {
    #[cfg(target_arch = "x86_64")]
    {
        #[cfg(target_feature = "bmi1")]
        unsafe {
            return core::arch::x86_64::_blsi_u64(x);
        }

        #[cfg(not(target_feature = "bmi1"))]
        {
            x & x.wrapping_neg()
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        x & x.wrapping_neg()
    }
}

/// Parallel bit extract (PEXT - BMI2 instruction).
///
/// Extracts bits from `src` according to `mask` and packs them into the
/// low bits. Critical for "fancy magic bitboards" - fast slider move
/// generation.
///
/// # Performance
/// - Hardware PEXT (Zen 3+, Intel Haswell+): ~3 cycles
/// - Software fallback: ~20-40 cycles
///
/// Note: AMD Zen 1/2 have slow microcode PEXT (~18 cycles), prefer fallback
/// there.
pub fn pext(src: u64, mask: u64) -> u64 {
    #[cfg(target_arch = "x86_64")]
    {
        #[cfg(target_feature = "bmi2")]
        unsafe {
            // BMI2 PEXT available - use hardware instruction
            core::arch::x86_64::_pext_u64(src, mask)
        }

        #[cfg(not(target_feature = "bmi2"))]
        {
            pext_software(src, mask)
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        pext_software(src, mask)
    }
}

/// Parallel bit extract (PEXT) - optimized for guaranteed non-zero mask.
///
/// Precondition: `mask != 0`. Enforced with debug_assert.
/// This version is preferred for hot paths where mask is always non-zero
/// (e.g., occupancy indexing with valid occupancy masks).
pub fn pext_nonzero(src: u64, mask: u64) -> u64 {
    debug_assert!(mask != 0, "pext_nonzero requires non-zero mask");
    pext(src, mask)
}

/// Software fallback for PEXT (used when BMI2 not available).
///
/// This is the "Kindergarten" bitboard approach - slower but portable.
#[allow(dead_code)]
fn pext_software(src: u64, mut mask: u64) -> u64 {
    let mut result = 0u64;
    let mut bb = 1u64;

    while mask != 0 {
        if src & mask & mask.wrapping_neg() != 0 {
            result |= bb;
        }
        mask &= mask.wrapping_sub(1);
        bb <<= 1;
    }

    result
}

/// Parallel bit deposit (PDEP - BMI2 instruction).
///
/// Inverse of PEXT: deposits bits from `src` into positions specified by
/// `mask`. Less commonly used in chess engines but available for completeness.
#[allow(dead_code)]
pub fn pdep(src: u64, mask: u64) -> u64 {
    #[cfg(target_arch = "x86_64")]
    {
        #[cfg(target_feature = "bmi2")]
        unsafe {
            core::arch::x86_64::_pdep_u64(src, mask)
        }

        #[cfg(not(target_feature = "bmi2"))]
        {
            pdep_software(src, mask)
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        pdep_software(src, mask)
    }
}

/// Software fallback for PDEP.
#[allow(dead_code)]
fn pdep_software(mut src: u64, mut mask: u64) -> u64 {
    let mut result = 0u64;

    while mask != 0 {
        let bit_pos = mask.trailing_zeros();
        let bit_mask = 1u64 << bit_pos;

        if src & 1 != 0 {
            result |= bit_mask;
        }

        src >>= 1;
        mask &= !bit_mask;
    }

    result
}

// ============================================================================
// SIMD / AVX Operations
// ============================================================================

/// SIMD vector type for 4x u64 values (256-bit AVX2 register).
///
/// Used for parallel operations on multiple bitboards simultaneously.
/// On non-AVX2 targets, operations fall back to sequential processing.
#[derive(Clone, Copy, Debug)]
#[repr(align(32))]
pub struct SimdU64x4 {
    pub data: [u64; 4],
}

impl SimdU64x4 {
    /// Create a new SIMD vector from 4 u64 values.
    pub fn new(a: u64, b: u64, c: u64, d: u64) -> Self {
        Self { data: [a, b, c, d] }
    }

    /// Create a SIMD vector with all elements set to the same value.
    pub fn splat(value: u64) -> Self {
        Self { data: [value; 4] }
    }

    /// Parallel population count: count bits in 4 bitboards simultaneously.
    ///
    /// Returns an array of 4 population counts.
    /// Uses AVX2 when available for maximum throughput.
    pub fn popcnt_parallel(self) -> [u32; 4] {
        #[cfg(all(
            target_arch = "x86_64",
            target_feature = "avx2",
            target_feature = "popcnt"
        ))]
        unsafe {
            // AVX2 path: use SIMD load + parallel popcnt
            [
                core::arch::x86_64::_popcnt64(self.data[0] as i64) as u32,
                core::arch::x86_64::_popcnt64(self.data[1] as i64) as u32,
                core::arch::x86_64::_popcnt64(self.data[2] as i64) as u32,
                core::arch::x86_64::_popcnt64(self.data[3] as i64) as u32,
            ]
        }

        #[cfg(not(all(
            target_arch = "x86_64",
            target_feature = "avx2",
            target_feature = "popcnt"
        )))]
        {
            [
                popcnt(self.data[0]),
                popcnt(self.data[1]),
                popcnt(self.data[2]),
                popcnt(self.data[3]),
            ]
        }
    }

    /// Parallel bitwise AND: compute self & other for all 4 elements.
    pub fn and(self, other: Self) -> Self {
        #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
        unsafe {
            let a = core::arch::x86_64::_mm256_loadu_si256(self.data.as_ptr() as *const _);
            let b = core::arch::x86_64::_mm256_loadu_si256(other.data.as_ptr() as *const _);
            let result = core::arch::x86_64::_mm256_and_si256(a, b);

            let mut out = Self { data: [0; 4] };
            core::arch::x86_64::_mm256_storeu_si256(out.data.as_mut_ptr() as *mut _, result);
            out
        }

        #[cfg(not(all(target_arch = "x86_64", target_feature = "avx2")))]
        {
            Self {
                data: [
                    self.data[0] & other.data[0],
                    self.data[1] & other.data[1],
                    self.data[2] & other.data[2],
                    self.data[3] & other.data[3],
                ],
            }
        }
    }

    /// Parallel bitwise OR: compute self | other for all 4 elements.
    pub fn or(self, other: Self) -> Self {
        #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
        unsafe {
            let a = core::arch::x86_64::_mm256_loadu_si256(self.data.as_ptr() as *const _);
            let b = core::arch::x86_64::_mm256_loadu_si256(other.data.as_ptr() as *const _);
            let result = core::arch::x86_64::_mm256_or_si256(a, b);

            let mut out = Self { data: [0; 4] };
            core::arch::x86_64::_mm256_storeu_si256(out.data.as_mut_ptr() as *mut _, result);
            out
        }

        #[cfg(not(all(target_arch = "x86_64", target_feature = "avx2")))]
        {
            Self {
                data: [
                    self.data[0] | other.data[0],
                    self.data[1] | other.data[1],
                    self.data[2] | other.data[2],
                    self.data[3] | other.data[3],
                ],
            }
        }
    }

    /// Parallel bitwise XOR: compute self ^ other for all 4 elements.
    pub fn xor(self, other: Self) -> Self {
        #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
        unsafe {
            let a = core::arch::x86_64::_mm256_loadu_si256(self.data.as_ptr() as *const _);
            let b = core::arch::x86_64::_mm256_loadu_si256(other.data.as_ptr() as *const _);
            let result = core::arch::x86_64::_mm256_xor_si256(a, b);

            let mut out = Self { data: [0; 4] };
            core::arch::x86_64::_mm256_storeu_si256(out.data.as_mut_ptr() as *mut _, result);
            out
        }

        #[cfg(not(all(target_arch = "x86_64", target_feature = "avx2")))]
        {
            Self {
                data: [
                    self.data[0] ^ other.data[0],
                    self.data[1] ^ other.data[1],
                    self.data[2] ^ other.data[2],
                    self.data[3] ^ other.data[3],
                ],
            }
        }
    }

    /// Parallel bitwise NOT: compute !self for all 4 elements.
    pub fn not(self) -> Self {
        #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
        unsafe {
            let a = core::arch::x86_64::_mm256_loadu_si256(self.data.as_ptr() as *const _);
            let ones = core::arch::x86_64::_mm256_set1_epi64x(-1);
            let result = core::arch::x86_64::_mm256_xor_si256(a, ones);

            let mut out = Self { data: [0; 4] };
            core::arch::x86_64::_mm256_storeu_si256(out.data.as_mut_ptr() as *mut _, result);
            out
        }

        #[cfg(not(all(target_arch = "x86_64", target_feature = "avx2")))]
        {
            Self {
                data: [!self.data[0], !self.data[1], !self.data[2], !self.data[3]],
            }
        }
    }

    /// Test if any element is non-zero.
    pub fn any_nonzero(self) -> bool {
        self.data[0] != 0 || self.data[1] != 0 || self.data[2] != 0 || self.data[3] != 0
    }

    /// Test if all elements are zero.
    pub fn all_zero(self) -> bool {
        !self.any_nonzero()
    }
}

/// SIMD vector for 8x i32 values (256-bit AVX2 register).
///
/// Used for parallel evaluation scoring, move ordering, etc.
#[derive(Clone, Copy, Debug)]
#[repr(align(32))]
pub struct SimdI32x8 {
    pub data: [i32; 8],
}

impl SimdI32x8 {
    /// Create a new SIMD vector from 8 i32 values.
    pub fn new(data: [i32; 8]) -> Self {
        Self { data }
    }

    /// Create a SIMD vector with all elements set to the same value.
    pub fn splat(value: i32) -> Self {
        Self { data: [value; 8] }
    }

    /// Parallel addition: add 8 integers simultaneously.
    fn add_impl(self, other: Self) -> Self {
        #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
        unsafe {
            let a = core::arch::x86_64::_mm256_loadu_si256(self.data.as_ptr() as *const _);
            let b = core::arch::x86_64::_mm256_loadu_si256(other.data.as_ptr() as *const _);
            let result = core::arch::x86_64::_mm256_add_epi32(a, b);

            let mut out = Self { data: [0; 8] };
            core::arch::x86_64::_mm256_storeu_si256(out.data.as_mut_ptr() as *mut _, result);
            out
        }

        #[cfg(not(all(target_arch = "x86_64", target_feature = "avx2")))]
        {
            Self {
                data: [
                    self.data[0] + other.data[0],
                    self.data[1] + other.data[1],
                    self.data[2] + other.data[2],
                    self.data[3] + other.data[3],
                    self.data[4] + other.data[4],
                    self.data[5] + other.data[5],
                    self.data[6] + other.data[6],
                    self.data[7] + other.data[7],
                ],
            }
        }
    }

    /// Parallel subtraction: subtract 8 integers simultaneously.
    fn sub_impl(self, other: Self) -> Self {
        #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
        unsafe {
            let a = core::arch::x86_64::_mm256_loadu_si256(self.data.as_ptr() as *const _);
            let b = core::arch::x86_64::_mm256_loadu_si256(other.data.as_ptr() as *const _);
            let result = core::arch::x86_64::_mm256_sub_epi32(a, b);

            let mut out = Self { data: [0; 8] };
            core::arch::x86_64::_mm256_storeu_si256(out.data.as_mut_ptr() as *mut _, result);
            out
        }

        #[cfg(not(all(target_arch = "x86_64", target_feature = "avx2")))]
        {
            Self {
                data: [
                    self.data[0] - other.data[0],
                    self.data[1] - other.data[1],
                    self.data[2] - other.data[2],
                    self.data[3] - other.data[3],
                    self.data[4] - other.data[4],
                    self.data[5] - other.data[5],
                    self.data[6] - other.data[6],
                    self.data[7] - other.data[7],
                ],
            }
        }
    }

    /// Horizontal sum: add all 8 elements together.
    pub fn horizontal_sum(self) -> i32 {
        #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
        unsafe {
            let vec = core::arch::x86_64::_mm256_loadu_si256(self.data.as_ptr() as *const _);
            // Horizontal add: reduce 8 elements to 4, then to 2, then to 1
            let sum1 = core::arch::x86_64::_mm256_hadd_epi32(vec, vec);
            let sum2 = core::arch::x86_64::_mm256_hadd_epi32(sum1, sum1);

            // Extract high and low 128-bit lanes and sum them
            let low = core::arch::x86_64::_mm256_castsi256_si128(sum2);
            let high = core::arch::x86_64::_mm256_extracti128_si256::<1>(sum2);
            let final_sum = core::arch::x86_64::_mm_add_epi32(low, high);

            core::arch::x86_64::_mm_cvtsi128_si32(final_sum)
        }

        #[cfg(not(all(target_arch = "x86_64", target_feature = "avx2")))]
        {
            self.data.iter().sum()
        }
    }

    /// Parallel maximum: find max of corresponding elements.
    pub fn max(self, other: Self) -> Self {
        #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
        unsafe {
            let a = core::arch::x86_64::_mm256_loadu_si256(self.data.as_ptr() as *const _);
            let b = core::arch::x86_64::_mm256_loadu_si256(other.data.as_ptr() as *const _);
            let result = core::arch::x86_64::_mm256_max_epi32(a, b);

            let mut out = Self { data: [0; 8] };
            core::arch::x86_64::_mm256_storeu_si256(out.data.as_mut_ptr() as *mut _, result);
            out
        }

        #[cfg(not(all(target_arch = "x86_64", target_feature = "avx2")))]
        {
            Self {
                data: [
                    self.data[0].max(other.data[0]),
                    self.data[1].max(other.data[1]),
                    self.data[2].max(other.data[2]),
                    self.data[3].max(other.data[3]),
                    self.data[4].max(other.data[4]),
                    self.data[5].max(other.data[5]),
                    self.data[6].max(other.data[6]),
                    self.data[7].max(other.data[7]),
                ],
            }
        }
    }

    /// Parallel minimum: find min of corresponding elements.
    pub fn min(self, other: Self) -> Self {
        #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
        unsafe {
            let a = core::arch::x86_64::_mm256_loadu_si256(self.data.as_ptr() as *const _);
            let b = core::arch::x86_64::_mm256_loadu_si256(other.data.as_ptr() as *const _);
            let result = core::arch::x86_64::_mm256_min_epi32(a, b);

            let mut out = Self { data: [0; 8] };
            core::arch::x86_64::_mm256_storeu_si256(out.data.as_mut_ptr() as *mut _, result);
            out
        }

        #[cfg(not(all(target_arch = "x86_64", target_feature = "avx2")))]
        {
            Self {
                data: [
                    self.data[0].min(other.data[0]),
                    self.data[1].min(other.data[1]),
                    self.data[2].min(other.data[2]),
                    self.data[3].min(other.data[3]),
                    self.data[4].min(other.data[4]),
                    self.data[5].min(other.data[5]),
                    self.data[6].min(other.data[6]),
                    self.data[7].min(other.data[7]),
                ],
            }
        }
    }
}

impl std::ops::Add for SimdI32x8 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        self.add_impl(other)
    }
}

impl std::ops::Sub for SimdI32x8 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        self.sub_impl(other)
    }
}

// ============================================================================
// Tests and Usage Examples
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_popcnt() {
        assert_eq!(popcnt(0), 0);
        assert_eq!(popcnt(1), 1);
        assert_eq!(popcnt(0xFF), 8);
        assert_eq!(popcnt(0xFFFF_FFFF_FFFF_FFFF), 64);
        assert_eq!(popcnt(0x8000_0000_0000_0001), 2); // Two corners
    }

    #[test]
    fn test_trailing_zeros() {
        assert_eq!(trailing_zeros(1), 0);
        assert_eq!(trailing_zeros(2), 1);
        assert_eq!(trailing_zeros(0x8000_0000), 31);
        assert_eq!(trailing_zeros(0x8000_0000_0000_0000), 63);
        assert_eq!(trailing_zeros(0), 64); // Edge case
    }

    #[test]
    fn test_leading_zeros() {
        assert_eq!(leading_zeros(1), 63);
        assert_eq!(leading_zeros(0x8000_0000_0000_0000), 0);
        assert_eq!(leading_zeros(0xFF), 56);
        assert_eq!(leading_zeros(0), 64); // Edge case
    }

    #[test]
    fn test_blsr() {
        assert_eq!(blsr(0b1010), 0b1000); // Clear LSB
        assert_eq!(blsr(0b0001), 0b0000);
        assert_eq!(blsr(0b1111), 0b1110);
        assert_eq!(blsr(0), 0);
    }

    #[test]
    fn test_blsi() {
        assert_eq!(blsi(0b1010), 0b0010); // Extract LSB
        assert_eq!(blsi(0b0001), 0b0001);
        assert_eq!(blsi(0b1111), 0b0001);
        assert_eq!(blsi(0), 0);
    }

    #[test]
    fn test_pext() {
        // Extract bits 0, 2, 4 from 0b10101
        let src = 0b10101;
        let mask = 0b10101;
        assert_eq!(pext(src, mask), 0b111);

        // More complex: extract alternating bits
        let src = 0xAAAA_AAAA_AAAA_AAAA; // All odd bits set
        let mask = 0xAAAA_AAAA_AAAA_AAAA; // Extract odd bits
        assert_eq!(pext(src, mask), 0xFFFF_FFFF); // Packed into lower 32 bits
    }

    #[test]
    fn test_pdep() {
        // Inverse of PEXT: deposit bits into mask positions
        let src = 0b111;
        let mask = 0b10101;
        assert_eq!(pdep(src, mask), 0b10101);

        // Round-trip test: pdep(pext(x, mask), mask) == (x & mask)
        let x = 0x123456789ABCDEF0;
        let mask = 0x00FF00FF00FF00FF;
        assert_eq!(pdep(pext(x, mask), mask), x & mask);
    }

    #[test]
    fn test_bitboard_iteration() {
        // Typical chess engine pattern: iterate through set bits
        let mut bb = 0b1101; // Bits at positions 0, 2, 3
        let mut squares = Vec::new();

        while bb != 0 {
            let sq = trailing_zeros(bb);
            squares.push(sq);
            bb = blsr(bb); // Clear the bit we just extracted
        }

        assert_eq!(squares, vec![0, 2, 3]);
    }

    #[test]
    fn test_simd_u64x4_popcnt() {
        let vec = SimdU64x4::new(0xFF, 0xFFFF, 0xFFFFFFFF, 0xFFFFFFFFFFFFFFFF);
        let counts = vec.popcnt_parallel();
        assert_eq!(counts, [8, 16, 32, 64]);
    }

    #[test]
    fn test_simd_u64x4_bitwise() {
        let a = SimdU64x4::new(0xF0F0, 0xFF00, 0xAAAA, 0x5555);
        let b = SimdU64x4::new(0x0F0F, 0x00FF, 0x5555, 0xAAAA);

        // Test AND
        let and_result = a.and(b);
        assert_eq!(and_result.data, [0, 0, 0, 0]);

        // Test OR
        let or_result = a.or(b);
        assert_eq!(or_result.data, [0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF]);

        // Test XOR
        let xor_result = a.xor(b);
        assert_eq!(xor_result.data, [0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF]);

        // Test NOT
        let not_a = a.not();
        assert_eq!(not_a.data, [!0xF0F0, !0xFF00, !0xAAAA, !0x5555]);
    }

    #[test]
    fn test_simd_u64x4_zero_checks() {
        let zeros = SimdU64x4::new(0, 0, 0, 0);
        assert!(zeros.all_zero());
        assert!(!zeros.any_nonzero());

        let one_nonzero = SimdU64x4::new(0, 1, 0, 0);
        assert!(!one_nonzero.all_zero());
        assert!(one_nonzero.any_nonzero());
    }

    #[test]
    fn test_simd_i32x8_arithmetic() {
        let a = SimdI32x8::new([1, 2, 3, 4, 5, 6, 7, 8]);
        let b = SimdI32x8::new([8, 7, 6, 5, 4, 3, 2, 1]);

        // Test addition
        let sum = a + b;
        assert_eq!(sum.data, [9, 9, 9, 9, 9, 9, 9, 9]);

        // Test subtraction
        let diff = a - b;
        assert_eq!(diff.data, [-7, -5, -3, -1, 1, 3, 5, 7]);

        // Test horizontal sum
        assert_eq!(a.horizontal_sum(), 36);
        assert_eq!(b.horizontal_sum(), 36);
    }

    #[test]
    fn test_simd_i32x8_minmax() {
        let a = SimdI32x8::new([10, 20, 30, 40, 50, 60, 70, 80]);
        let b = SimdI32x8::new([80, 70, 60, 50, 40, 30, 20, 10]);

        // Test max
        let max_result = a.max(b);
        assert_eq!(max_result.data, [80, 70, 60, 50, 50, 60, 70, 80]);

        // Test min
        let min_result = a.min(b);
        assert_eq!(min_result.data, [10, 20, 30, 40, 40, 30, 20, 10]);
    }

    #[test]
    fn test_simd_i32x8_splat() {
        let vec = SimdI32x8::splat(42);
        assert_eq!(vec.data, [42, 42, 42, 42, 42, 42, 42, 42]);
        assert_eq!(vec.horizontal_sum(), 336);
    }
}
