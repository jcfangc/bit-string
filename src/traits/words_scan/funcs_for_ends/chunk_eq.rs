//! Single-purpose SIMD helper: are all words in a chunk equal to `FILL`?
//!
//! This is the *only* SIMD primitive needed by leading-/trailing-zero
//! counting.  There is no lane-scanning, no dispatch table — just a
//! fast equality check that keeps the hot path at 1–2 instructions.

// ── Compile-time LANES constant ──────────────────────────────────────
// In default mode (no compile-time-dispatch), LANES is pinned to the
// non-AVX2 maximum because the AVX2 leading/trailing paths inline raw
// intrinsics directly and never call chunk_eq.  In compile-time-dispatch
// mode LANES tracks the selected target feature.

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    not(feature = "compile-time-dispatch")
))]
pub(crate) const LANES: usize = 2; // SSE2 baseline

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    feature = "compile-time-dispatch",
    target_feature = "avx2"
))]
pub(crate) const LANES: usize = 4;

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    feature = "compile-time-dispatch",
    not(target_feature = "avx2")
))]
pub(crate) const LANES: usize = 2;

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
pub(crate) const LANES: usize = 2;

#[cfg(not(any(
    all(
        any(target_arch = "x86", target_arch = "x86_64"),
        not(feature = "compile-time-dispatch")
    ),
    all(
        any(target_arch = "x86", target_arch = "x86_64"),
        feature = "compile-time-dispatch"
    ),
    all(target_arch = "aarch64", target_feature = "neon"),
)))]
pub(crate) const LANES: usize = 1;

pub(crate) const LANES_2X: usize = LANES * 2;

// ═══════════════════════════════════════════════════════════════════════
// Runtime CPUID cache (default mode only)
// ═══════════════════════════════════════════════════════════════════════

#[cfg(all(
    not(feature = "compile-time-dispatch"),
    any(target_arch = "x86", target_arch = "x86_64")
))]
mod runtime {
    use core::sync::atomic::{AtomicU8, Ordering};

    const UNINIT: u8 = 0;
    const AVX2: u8 = 1;
    const SSE2: u8 = 2;

    static DETECTED: AtomicU8 = AtomicU8::new(UNINIT);

    #[cold]
    fn detect() -> u8 {
        #[cfg(target_arch = "x86_64")]
        {
            let leaf1 = unsafe { core::arch::x86_64::__cpuid_count(1, 0) };
            let leaf7 = unsafe { core::arch::x86_64::__cpuid_count(7, 0) };
            let has_avx2 = leaf7.ebx & (1 << 5) != 0;
            let has_sse2 = leaf1.edx & (1 << 26) != 0;
            let backend = if has_avx2 {
                AVX2
            } else if has_sse2 {
                SSE2
            } else {
                UNINIT
            };
            DETECTED.store(backend, Ordering::Relaxed);
            backend
        }
        #[cfg(target_arch = "x86")]
        {
            let leaf1 = unsafe { core::arch::x86::__cpuid_count(1, 0) };
            let leaf7 = unsafe { core::arch::x86::__cpuid_count(7, 0) };
            let has_avx2 = leaf7.ebx & (1 << 5) != 0;
            let has_sse2 = leaf1.edx & (1 << 26) != 0;
            let backend = if has_avx2 {
                AVX2
            } else if has_sse2 {
                SSE2
            } else {
                UNINIT
            };
            DETECTED.store(backend, Ordering::Relaxed);
            backend
        }
    }

    #[inline(always)]
    pub(super) fn has_avx2() -> bool {
        let b = DETECTED.load(Ordering::Relaxed);
        if b != UNINIT {
            return b == AVX2;
        }
        detect() == AVX2
    }

    #[inline(always)]
    pub(super) fn has_sse2() -> bool {
        let b = DETECTED.load(Ordering::Relaxed);
        if b != UNINIT {
            return b >= SSE2;
        }
        detect() >= SSE2
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Dispatch function
// ═══════════════════════════════════════════════════════════════════════

/// Returns `true` when all `LANES` u64 values at `ptr` equal `FILL`.
///
/// Dispatches to the best available SIMD backend at runtime (default) or
/// compile time (`compile-time-dispatch` feature).
///
/// # Safety
///
/// `ptr` must be valid for reads of `LANES` u64 values on the caller's
/// target.  The caller must ensure the selected backend's instructions
/// are available when using runtime dispatch.
#[inline]
pub(crate) unsafe fn chunk_eq<const FILL: u64>(ptr: *const u64) -> bool {
    // ── Default: runtime SIMD detection ─────────────────────────
    #[cfg(not(feature = "compile-time-dispatch"))]
    {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            if runtime::has_avx2() {
                return unsafe { avx2::chunk_eq::<FILL>(ptr) };
            }
            if runtime::has_sse2() {
                return unsafe { sse2::chunk_eq::<FILL>(ptr) };
            }
        }
        #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
        {
            return unsafe { neon::chunk_eq::<FILL>(ptr) };
        }
        #[allow(unused)]
        unsafe {
            scalar::chunk_eq::<FILL>(ptr)
        }
    }

    // ── compile-time-dispatch: pure #[cfg] cascade ──────────────
    #[cfg(feature = "compile-time-dispatch")]
    {
        #[cfg(all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "avx2"
        ))]
        {
            return unsafe { avx2::chunk_eq::<FILL>(ptr) };
        }

        #[cfg(all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "sse2",
            not(target_feature = "avx2")
        ))]
        {
            return unsafe { sse2::chunk_eq::<FILL>(ptr) };
        }

        #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
        {
            return unsafe { neon::chunk_eq::<FILL>(ptr) };
        }

        #[allow(unused)]
        unsafe {
            scalar::chunk_eq::<FILL>(ptr)
        }
    }
}

/// 2×‑unrolled chunk equality: checks two adjacent chunks.
///
/// # Safety
///
/// `ptr` must be valid for reads of `LANES_2X` u64 values.
#[inline]
pub(crate) unsafe fn chunk_eq_2x<const FILL: u64>(ptr: *const u64) -> bool {
    // SAFETY: `ptr` is valid for `LANES_2X` = 2×LANES u64 reads
    // per caller guarantee.  Both `ptr` and `ptr.add(LANES)` are
    // within the promised range.
    unsafe { chunk_eq::<FILL>(ptr) && chunk_eq::<FILL>(ptr.add(LANES)) }
}

// ═══════════════════════════════════════════════════════════════════════
// AVX2 — 256-bit / 4-lane
// ═══════════════════════════════════════════════════════════════════════

// Compiled on x86 in default mode (available via runtime dispatch) and
// in compile-time-dispatch mode when AVX2 is the target feature.
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    any(not(feature = "compile-time-dispatch"), target_feature = "avx2")
))]
mod avx2 {
    // The AVX2 leading/trailing paths inline raw intrinsics directly, so
    // this module may be unused in default mode.
    #![cfg_attr(not(feature = "compile-time-dispatch"), allow(dead_code))]
    #[cfg(target_arch = "x86")]
    use core::arch::x86::{
        __m256i, _mm256_loadu_si256, _mm256_set1_epi64x, _mm256_testz_si256, _mm256_xor_si256,
    };
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m256i, _mm256_loadu_si256, _mm256_set1_epi64x, _mm256_testz_si256, _mm256_xor_si256,
    };

    /// # Safety
    ///
    /// `ptr` must be valid for reads of 4 u64 values.  Caller must
    /// ensure AVX2 is available.
    #[inline]
    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn chunk_eq<const FILL: u64>(ptr: *const u64) -> bool {
        // SAFETY: caller guarantees target_feature `avx2` is available and
        // `ptr` is valid for 4 u64 reads.
        unsafe {
            let data = _mm256_loadu_si256(ptr.cast::<__m256i>());
            if FILL == 0 {
                _mm256_testz_si256(data, data) != 0
            } else {
                let fill_vec = _mm256_set1_epi64x(FILL as i64);
                let xor = _mm256_xor_si256(data, fill_vec);
                _mm256_testz_si256(xor, xor) != 0
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SSE2 — 128-bit / 2-lane (x86_64 baseline)
// ═══════════════════════════════════════════════════════════════════════

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    any(
        not(feature = "compile-time-dispatch"),
        all(target_feature = "sse2", not(target_feature = "avx2"))
    )
))]
mod sse2 {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::{
        __m128i, _mm_cmpeq_epi32, _mm_loadu_si128, _mm_movemask_epi8, _mm_set1_epi64x,
        _mm_setzero_si128, _mm_xor_si128,
    };
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m128i, _mm_cmpeq_epi32, _mm_loadu_si128, _mm_movemask_epi8, _mm_set1_epi64x,
        _mm_setzero_si128, _mm_xor_si128,
    };

    /// Returns `true` when all 2 u64 values at `ptr` equal `FILL`.
    ///
    /// Uses the SSE2 baseline (pcmeq + pmovmskb).  On x86-64 SSE2 is
    /// always available without special compile flags.
    ///
    /// # Safety
    ///
    /// `ptr` must be valid for reads of 2 u64 values.
    #[inline]
    #[target_feature(enable = "sse2")]
    pub(super) unsafe fn chunk_eq<const FILL: u64>(ptr: *const u64) -> bool {
        // SAFETY: caller guarantees `ptr` is valid for 2 u64 reads.
        // SSE2 is available per `#[target_feature]`.
        unsafe {
            let data = _mm_loadu_si128(ptr.cast::<__m128i>());
            let zero = _mm_setzero_si128();
            if FILL == 0 {
                // data XOR 0 == data; check that all 128 bits are zero.
                let cmp = _mm_cmpeq_epi32(data, zero);
                _mm_movemask_epi8(cmp) == 0xFFFF
            } else {
                let fill_vec = _mm_set1_epi64x(FILL as i64);
                let xor = _mm_xor_si128(data, fill_vec);
                let cmp = _mm_cmpeq_epi32(xor, zero);
                _mm_movemask_epi8(cmp) == 0xFFFF
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// NEON — 128-bit / 2-lane
// ═══════════════════════════════════════════════════════════════════════

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
mod neon {
    use core::arch::aarch64::{uint64x2_t, vceqq_u64, vdupq_n_u64, vgetq_lane_u64, vld1q_u64};

    /// # Safety
    ///
    /// `ptr` must be valid for reads of 2 u64 values.  Caller must
    /// ensure NEON is available.
    #[inline]
    #[target_feature(enable = "neon")]
    pub(super) unsafe fn chunk_eq<const FILL: u64>(ptr: *const u64) -> bool {
        // SAFETY: caller guarantees target_feature `neon` is available and
        // `ptr` is valid for 2 u64 reads.
        unsafe {
            let data = vld1q_u64(ptr);
            let cmp = vceqq_u64(data, vdupq_n_u64(FILL));
            vgetq_lane_u64(cmp, 0) != 0 && vgetq_lane_u64(cmp, 1) != 0
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Scalar fallback — always compiled
// ═══════════════════════════════════════════════════════════════════════

#[allow(unused)]
mod scalar {
    #[inline]
    pub(super) unsafe fn chunk_eq<const FILL: u64>(ptr: *const u64) -> bool {
        // SAFETY: caller guarantees `ptr` is valid for a u64 read.
        unsafe { *ptr == FILL }
    }
}
