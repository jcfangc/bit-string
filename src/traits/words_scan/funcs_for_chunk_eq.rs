//! Single-purpose SIMD helper: are all words in a chunk equal to `FILL`?
//!
//! This is the *only* SIMD primitive needed by leading-/trailing-zero
//! counting.  There is no lane-scanning, no dispatch table — just a
//! fast equality check that keeps the hot path at 1–2 instructions.

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
mod imp {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::{
        __m256i, _mm256_loadu_si256, _mm256_set1_epi64x, _mm256_testz_si256, _mm256_xor_si256,
    };
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m256i, _mm256_loadu_si256, _mm256_set1_epi64x, _mm256_testz_si256, _mm256_xor_si256,
    };

    pub(crate) const LANES: usize = 4;

    /// Returns `true` when all `LANES` u64 values at `ptr` equal `FILL`.
    ///
    /// # Safety
    ///
    /// `ptr` must be valid for reads of `LANES` u64 values.  Caller must
    /// ensure AVX2 is available.
    #[inline]
    #[target_feature(enable = "avx2")]
    pub(crate) unsafe fn chunk_eq<const FILL: u64>(ptr: *const u64) -> bool {
        // SAFETY: caller guarantees target_feature `avx2` is available and
        // `ptr` is valid for `LANES` u64 reads.
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

// x86/x86-64 without AVX2 — use SSE2 (baseline on x86-64, optionally
// available on i686).  SSE2 provides 128-bit / 2-lane equality via
// pcmeqepi32 + pmovmskb, always available on x86-64 without any compile
// flags.
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "sse2",
    not(target_feature = "avx2")
))]
mod imp {
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

    pub(crate) const LANES: usize = 2;

    /// Returns `true` when all `LANES` u64 values at `ptr` equal `FILL`.
    ///
    /// Uses the SSE2 baseline (pcmeq + pmovmskb).  On x86-64 SSE2 is
    /// always available without special compile flags.
    ///
    /// # Safety
    ///
    /// `ptr` must be valid for reads of `LANES` u64 values.
    #[inline]
    #[target_feature(enable = "sse2")]
    pub(crate) unsafe fn chunk_eq<const FILL: u64>(ptr: *const u64) -> bool {
        // SAFETY: caller guarantees `ptr` is valid for `LANES` u64 reads.
        // SSE2 is available per `#[cfg(target_feature = "sse2")]`.
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

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
mod imp {
    use core::arch::aarch64::{uint64x2_t, vceqq_u64, vdupq_n_u64, vgetq_lane_u64, vld1q_u64};

    pub(crate) const LANES: usize = 2;

    /// # Safety
    ///
    /// `ptr` must be valid for reads of `LANES` u64 values.  Caller must
    /// ensure NEON is available.
    #[inline]
    #[target_feature(enable = "neon")]
    pub(crate) unsafe fn chunk_eq<const FILL: u64>(ptr: *const u64) -> bool {
        // SAFETY: caller guarantees target_feature `neon` is available and
        // `ptr` is valid for `LANES` u64 reads.
        unsafe {
            let data = vld1q_u64(ptr);
            let cmp = vceqq_u64(data, vdupq_n_u64(FILL));
            vgetq_lane_u64(cmp, 0) != 0 && vgetq_lane_u64(cmp, 1) != 0
        }
    }
}

// Scalar fallback — no SIMD feature available.
#[cfg(not(any(
    all(
        any(target_arch = "x86", target_arch = "x86_64"),
        any(target_feature = "avx2", target_feature = "sse2")
    ),
    all(target_arch = "aarch64", target_feature = "neon"),
)))]
mod imp {
    pub(crate) const LANES: usize = 1;

    #[inline]
    pub(crate) unsafe fn chunk_eq<const FILL: u64>(ptr: *const u64) -> bool {
        // SAFETY: caller guarantees `ptr` is valid for a u64 read.
        unsafe { *ptr == FILL }
    }
}

pub(crate) use imp::{LANES, chunk_eq};

// ═══════════════════════════════════════════════════════════════════════
// 2×‑unrolled and aligned chunk‑eq primitives.
// Each `#[cfg]` block below is mutually exclusive — exactly one is
// compiled per target tuple.  Callers use the bare name (e.g.
// `chunk_eq_2x::<FILL>(ptr)`) without a module prefix.
// ═══════════════════════════════════════════════════════════════════════

// ── AVX2 ─────────────────────────────────────────────────────────────
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
mod chunk_eq_avx2_extras {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::{
        __m256i, _mm256_load_si256, _mm256_loadu_si256, _mm256_set1_epi64x, _mm256_testz_si256,
        _mm256_xor_si256,
    };
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m256i, _mm256_load_si256, _mm256_loadu_si256, _mm256_set1_epi64x, _mm256_testz_si256,
        _mm256_xor_si256,
    };

    pub(crate) const LANES_2X: usize = 8;

    #[inline]
    #[target_feature(enable = "avx2")]
    pub(crate) unsafe fn chunk_eq_2x<const FILL: u64>(ptr: *const u64) -> bool {
        unsafe {
            let d0 = _mm256_loadu_si256(ptr.cast::<__m256i>());
            let d1 = _mm256_loadu_si256(ptr.add(4).cast::<__m256i>());
            if FILL == 0 {
                _mm256_testz_si256(d0, d0) != 0 && _mm256_testz_si256(d1, d1) != 0
            } else {
                let fill_vec = _mm256_set1_epi64x(FILL as i64);
                let x0 = _mm256_xor_si256(d0, fill_vec);
                let x1 = _mm256_xor_si256(d1, fill_vec);
                _mm256_testz_si256(x0, x0) != 0 && _mm256_testz_si256(x1, x1) != 0
            }
        }
    }

    #[inline]
    #[target_feature(enable = "avx2")]
    pub(crate) unsafe fn chunk_eq_aligned<const FILL: u64>(ptr: *const u64) -> bool {
        unsafe {
            let data = _mm256_load_si256(ptr.cast::<__m256i>());
            if FILL == 0 {
                _mm256_testz_si256(data, data) != 0
            } else {
                let fill_vec = _mm256_set1_epi64x(FILL as i64);
                let xor = _mm256_xor_si256(data, fill_vec);
                _mm256_testz_si256(xor, xor) != 0
            }
        }
    }

    #[inline]
    #[target_feature(enable = "avx2")]
    pub(crate) unsafe fn chunk_eq_2x_aligned<const FILL: u64>(ptr: *const u64) -> bool {
        unsafe {
            let d0 = _mm256_load_si256(ptr.cast::<__m256i>());
            let d1 = _mm256_load_si256(ptr.add(4).cast::<__m256i>());
            if FILL == 0 {
                _mm256_testz_si256(d0, d0) != 0 && _mm256_testz_si256(d1, d1) != 0
            } else {
                let fill_vec = _mm256_set1_epi64x(FILL as i64);
                let x0 = _mm256_xor_si256(d0, fill_vec);
                let x1 = _mm256_xor_si256(d1, fill_vec);
                _mm256_testz_si256(x0, x0) != 0 && _mm256_testz_si256(x1, x1) != 0
            }
        }
    }
}
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
pub(crate) use chunk_eq_avx2_extras::{
    LANES_2X, chunk_eq_2x, chunk_eq_2x_aligned, chunk_eq_aligned,
};

// ── SSE2 (x86 baseline, no AVX2) ──────────────────────────────────────
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "sse2",
    not(target_feature = "avx2")
))]
mod chunk_eq_sse2_extras {
    use super::imp::LANES;

    pub(crate) const LANES_2X: usize = LANES * 2; // 4

    #[inline]
    pub(crate) unsafe fn chunk_eq_2x<const FILL: u64>(ptr: *const u64) -> bool {
        use super::imp::chunk_eq;
        unsafe { chunk_eq::<FILL>(ptr) && chunk_eq::<FILL>(ptr.add(LANES)) }
    }
}
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "sse2",
    not(target_feature = "avx2")
))]
pub(crate) use chunk_eq_sse2_extras::LANES_2X;

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "sse2",
    not(target_feature = "avx2")
))]
pub(crate) use chunk_eq_sse2_extras::chunk_eq_2x;

// ── NEON (aarch64) ────────────────────────────────────────────────────
#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
mod chunk_eq_neon_extras {
    use super::imp::LANES;

    pub(crate) const LANES_2X: usize = LANES * 2; // 4

    #[inline]
    pub(crate) unsafe fn chunk_eq_2x<const FILL: u64>(ptr: *const u64) -> bool {
        use super::imp::chunk_eq;
        unsafe { chunk_eq::<FILL>(ptr) && chunk_eq::<FILL>(ptr.add(LANES)) }
    }
}
#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
pub(crate) use chunk_eq_neon_extras::LANES_2X;

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
pub(crate) use chunk_eq_neon_extras::chunk_eq_2x;

// ── Scalar fallback ───────────────────────────────────────────────────
#[cfg(not(any(
    all(
        any(target_arch = "x86", target_arch = "x86_64"),
        any(target_feature = "avx2", target_feature = "sse2")
    ),
    all(target_arch = "aarch64", target_feature = "neon"),
)))]
mod chunk_eq_scalar_extras {
    use super::imp::LANES;

    pub(crate) const LANES_2X: usize = LANES * 2; // 2

    #[inline]
    pub(crate) unsafe fn chunk_eq_2x<const FILL: u64>(ptr: *const u64) -> bool {
        use super::imp::chunk_eq;
        unsafe { chunk_eq::<FILL>(ptr) && chunk_eq::<FILL>(ptr.add(LANES)) }
    }
}
#[cfg(not(any(
    all(
        any(target_arch = "x86", target_arch = "x86_64"),
        any(target_feature = "avx2", target_feature = "sse2")
    ),
    all(target_arch = "aarch64", target_feature = "neon"),
)))]
pub(crate) use chunk_eq_scalar_extras::LANES_2X;

#[cfg(not(any(
    all(
        any(target_arch = "x86", target_arch = "x86_64"),
        any(target_feature = "avx2", target_feature = "sse2")
    ),
    all(target_arch = "aarch64", target_feature = "neon"),
)))]
pub(crate) use chunk_eq_scalar_extras::chunk_eq_2x;
