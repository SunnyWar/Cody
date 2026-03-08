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
