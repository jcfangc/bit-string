//! SIMD word-level equality — compares LANES u64 words at once
//! via `cmpeq` + `movemask`.

use crate::SMALL_WORDS;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Returns `true` if the first `count` words of `src` match `other`.
///
/// Dispatches to the best available SIMD backend at compile time.
/// Short inputs fall back to scalar.
#[inline]
pub(super) fn eq_words_aligned(src: &[u64], other: &[u64], count: usize) -> bool {
    if count < SMALL_WORDS {
        for i in 0..count {
            if src[i] != other[i] {
                return false;
            }
        }
        return true;
    }

    // ── Default: runtime SIMD detection ─────────────────────────
    #[cfg(not(feature = "compile-time-dispatch"))]
    {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            let f = crate::cpuid::features();
            if f.avx2 {
                // SAFETY: `src`/`other` are valid for `count` words. Backend was selected via CPUID verification.
                return unsafe { avx2::eq_words(src, other, count) };
            }
            if f.sse41 {
                // SAFETY: `src`/`other` are valid for `count` words. Backend was selected via CPUID verification.
                return unsafe { sse41::eq_words(src, other, count) };
            }
        }
        #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
        {
            // SAFETY: `src`/`other` are valid for `count` words. Backend feature is enabled by the `#[cfg]` gate on this block.
            return unsafe { neon::eq_words(src, other, count) };
        }
        #[allow(unused)]
        {
            for i in 0..count {
                if src[i] != other[i] {
                    return false;
                }
            }
            true
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
            // SAFETY: `src`/`other` are valid for `count` words. Backend feature is guaranteed by compile-time `#[cfg]` gate.
            return unsafe { avx2::eq_words(src, other, count) };
        }

        #[cfg(all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "sse4.1",
            not(target_feature = "avx2")
        ))]
        {
            // SAFETY: `src`/`other` are valid for `count` words. Backend feature is guaranteed by compile-time `#[cfg]` gate.
            return unsafe { sse41::eq_words(src, other, count) };
        }

        #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
        {
            // SAFETY: `src`/`other` are valid for `count` words. Backend feature is enabled by the `#[cfg]` gate on this block.
            return unsafe { neon::eq_words(src, other, count) };
        }

        #[allow(unused)]
        {
            for i in 0..count {
                if src[i] != other[i] {
                    return false;
                }
            }
            true
        }
    }
}

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2 {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::{__m256i, _mm256_cmpeq_epi64, _mm256_loadu_si256, _mm256_movemask_pd};
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{__m256i, _mm256_cmpeq_epi64, _mm256_loadu_si256, _mm256_movemask_pd};

    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn eq_words(src: &[u64], other: &[u64], len: usize) -> bool {
        let mut i = 0;
        while i + 4 <= len {
            // SAFETY: `#[target_feature(enable = "avx2")]` ensures AVX2 is enabled.
            // Pointers `src` and `other` are valid for `len` elements (guaranteed by caller).
            let a = unsafe { _mm256_loadu_si256(src.as_ptr().add(i).cast::<__m256i>()) };
            // SAFETY: same as above; load from `other`.
            let b = unsafe { _mm256_loadu_si256(other.as_ptr().add(i).cast::<__m256i>()) };
            // SAFETY: `_mm256_cmpeq_epi64` and `_mm256_movemask_pd` are pure register operations; AVX2 is enabled by `#[target_feature]`.
            let cmp = unsafe { _mm256_cmpeq_epi64(a, b) };
            // SAFETY: same as above
            if unsafe { _mm256_movemask_pd(core::mem::transmute(cmp)) } as u32 != 0b1111 {
                return false;
            }
            i += 4;
        }
        while i < len {
            if src[i] != other[i] {
                return false;
            }
            i += 1;
        }
        true
    }
}

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse41 {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::{__m128i, _mm_cmpeq_epi64, _mm_loadu_si128, _mm_movemask_epi8};
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{__m128i, _mm_cmpeq_epi64, _mm_loadu_si128, _mm_movemask_epi8};

    #[target_feature(enable = "sse4.1")]
    pub(super) unsafe fn eq_words(src: &[u64], other: &[u64], len: usize) -> bool {
        let mut i = 0;
        while i + 2 <= len {
            // SAFETY: `#[target_feature(enable = "sse4.1")]` ensures SSE4.1 is enabled.
            // Pointers `src` and `other` are valid for `len` elements (guaranteed by caller).
            let a = unsafe { _mm_loadu_si128(src.as_ptr().add(i).cast::<__m128i>()) };
            // SAFETY: same as above; load from `other`.
            let b = unsafe { _mm_loadu_si128(other.as_ptr().add(i).cast::<__m128i>()) };
            // SAFETY: `_mm_cmpeq_epi64` and `_mm_movemask_epi8` are pure register operations; SSE4.1 is enabled by `#[target_feature]`.
            let cmp = unsafe { _mm_cmpeq_epi64(a, b) };
            // SAFETY: same as above
            if unsafe { _mm_movemask_epi8(cmp) } as u32 != 0xFFFF {
                return false;
            }
            i += 2;
        }
        while i < len {
            if src[i] != other[i] {
                return false;
            }
            i += 1;
        }
        true
    }
}

#[allow(unused)]
#[cfg(target_arch = "aarch64")]
mod neon {
    use core::arch::aarch64::{uint64x2_t, vceqq_u64, vgetq_lane_u64, vld1q_u64};

    #[target_feature(enable = "neon")]
    pub(super) unsafe fn eq_words(src: &[u64], other: &[u64], len: usize) -> bool {
        let mut i = 0;
        while i + 2 <= len {
            // SAFETY: `#[target_feature(enable = "neon")]` ensures NEON is enabled.
            // Pointers `src` and `other` are valid for `len` elements (guaranteed by caller).
            let a = unsafe { vld1q_u64(src.as_ptr().add(i)) };
            // SAFETY: same as above; load from `other`.
            let b = unsafe { vld1q_u64(other.as_ptr().add(i)) };
            // SAFETY: `vceqq_u64` and `vgetq_lane_u64` are pure register operations; NEON is enabled by `#[target_feature]`.
            let cmp = unsafe { vceqq_u64(a, b) };
            // SAFETY: same as above
            if unsafe { vgetq_lane_u64(cmp, 0) } == 0 || unsafe { vgetq_lane_u64(cmp, 1) } == 0 {
                return false;
            }
            i += 2;
        }
        while i < len {
            if src[i] != other[i] {
                return false;
            }
            i += 1;
        }
        true
    }
}

#[cfg(test)]
mod tests_for_backend_equivalence;
