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
#[inline(always)]
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
#[inline(always)]
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
#[inline(always)]
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
#[inline(always)]
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
#[inline(always)]
pub fn trailing_zeros(x: u64) -> u32 {
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
#[inline(always)]
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
#[inline(always)]
pub fn blsr(x: u64) -> u64 {
    #[cfg(target_arch = "x86_64")]
    {
        #[cfg(target_feature = "bmi1")]
        unsafe {
            return core::arch::x86_64::_blsr_u64(x);
        }

        #[cfg(not(target_feature = "bmi1"))]
        {
            x & x.wrapping_sub(1)
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        x & x.wrapping_sub(1)
    }
}

/// Extract the least significant set bit (x & -x).
///
/// Isolates a single bit for masking operations.
/// Example: get the lowest piece on a bitboard without removing it.
#[inline(always)]
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
#[inline(always)]
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

/// Software fallback for PEXT (used when BMI2 not available).
///
/// This is the "Kindergarten" bitboard approach - slower but portable.
#[inline(always)]
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
#[inline(always)]
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
#[inline(always)]
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
}
