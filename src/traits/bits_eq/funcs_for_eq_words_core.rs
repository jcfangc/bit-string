//! SIMD word-level equality — `cmpeq` + `movemask`.
//!
//! Single entry point `eq_words::<HS_WORD_ALIGNED>` dispatches to the
//! aligned or unaligned backend based on the const-generic.

use crate::SMALL_WORDS;
use crate::WORD_BITS;

// ---------------------------------------------------------------------------
// Unified entry point
// ---------------------------------------------------------------------------

/// Returns `true` if the first `full_words` words of `src` match `needle`.
///
/// `HS_WORD_ALIGNED` guarantees `haystack_shift == 0`, eliminating the
/// unaligned backend at compile time.
#[inline]
pub(super) fn eq_words<const HS_WORD_ALIGNED: bool>(
    src: &[u64],
    needle: &[u64],
    full_words: usize,
    haystack_shift: usize,
) -> bool {
    if HS_WORD_ALIGNED || haystack_shift == 0 {
        eq_words_aligned(src, needle, full_words)
    } else {
        eq_words_unaligned(src, needle, full_words, haystack_shift)
    }
}

// ---------------------------------------------------------------------------
// Aligned entry — direct word comparison
// ---------------------------------------------------------------------------

#[inline]
fn eq_words_aligned(src: &[u64], other: &[u64], count: usize) -> bool {
    if count < SMALL_WORDS {
        for i in 0..count {
            if src[i] != other[i] {
                return false;
            }
        }
        return true;
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    {
        return unsafe { avx2::eq_words(src, other, count) };
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse4.1",
        not(target_feature = "avx2")
    ))]
    {
        return unsafe { sse41::eq_words(src, other, count) };
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
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

// ---------------------------------------------------------------------------
// Unaligned entry — shifted-window comparison
// ---------------------------------------------------------------------------

#[inline]
fn eq_words_unaligned(src: &[u64], other: &[u64], count: usize, shift: usize) -> bool {
    debug_assert!(shift > 0 && shift < WORD_BITS);

    if count < SMALL_WORDS {
        for i in 0..count {
            let w0 = src[i];
            let w1 = src[i + 1];
            if ((w0 >> shift) | (w1 << (WORD_BITS - shift))) != other[i] {
                return false;
            }
        }
        return true;
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    {
        return unsafe { avx2::eq_words_unaligned(src, other, count, shift) };
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse4.1",
        not(target_feature = "avx2")
    ))]
    {
        return unsafe { sse41::eq_words_unaligned(src, other, count, shift) };
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        return unsafe { neon::eq_words_unaligned(src, other, count, shift) };
    }

    #[allow(unused)]
    {
        for i in 0..count {
            let w0 = src[i];
            let w1 = src[i + 1];
            if ((w0 >> shift) | (w1 << (WORD_BITS - shift))) != other[i] {
                return false;
            }
        }
        true
    }
}

// ===========================================================================
// AVX2 backend
// ===========================================================================

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2 {
    use crate::WORD_BITS;

    #[cfg(target_arch = "x86")]
    use core::arch::x86::{
        __m128i, __m256i, _mm_set1_epi64x, _mm256_cmpeq_epi64, _mm256_loadu_si256,
        _mm256_movemask_pd, _mm256_or_si256, _mm256_sll_epi64, _mm256_srl_epi64,
    };
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m128i, __m256i, _mm_set1_epi64x, _mm256_cmpeq_epi64, _mm256_loadu_si256,
        _mm256_movemask_pd, _mm256_or_si256, _mm256_sll_epi64, _mm256_srl_epi64,
    };

    // -- aligned ------------------------------------------------------------

    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn eq_words(src: &[u64], other: &[u64], len: usize) -> bool {
        let mut i = 0;
        while i + 4 <= len {
            let a = _mm256_loadu_si256(src.as_ptr().add(i).cast::<__m256i>());
            let b = _mm256_loadu_si256(other.as_ptr().add(i).cast::<__m256i>());
            let cmp = _mm256_cmpeq_epi64(a, b);
            if _mm256_movemask_pd(core::mem::transmute(cmp)) as u32 != 0b1111 {
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

    // -- unaligned ----------------------------------------------------------

    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn eq_words_unaligned(
        src: &[u64],
        other: &[u64],
        len: usize,
        shift: usize,
    ) -> bool {
        let count_lo = _mm_set1_epi64x(shift as i64);
        let count_hi = _mm_set1_epi64x((WORD_BITS - shift) as i64);
        let mut i = 0;
        while i + 4 <= len {
            let w0 = _mm256_loadu_si256(src.as_ptr().add(i).cast::<__m256i>());
            let w1 = _mm256_loadu_si256(src.as_ptr().add(i + 1).cast::<__m256i>());
            let lo = _mm256_srl_epi64(w0, count_lo);
            let hi = _mm256_sll_epi64(w1, count_hi);
            let window = _mm256_or_si256(lo, hi);
            let b = _mm256_loadu_si256(other.as_ptr().add(i).cast::<__m256i>());
            let cmp = _mm256_cmpeq_epi64(window, b);
            if _mm256_movemask_pd(core::mem::transmute(cmp)) as u32 != 0b1111 {
                return false;
            }
            i += 4;
        }
        while i < len {
            let w0 = src[i];
            let w1 = src[i + 1];
            if ((w0 >> shift) | (w1 << (WORD_BITS - shift))) != other[i] {
                return false;
            }
            i += 1;
        }
        true
    }
}

// ===========================================================================
// SSE4.1 backend
// ===========================================================================

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse41 {
    use crate::WORD_BITS;

    #[cfg(target_arch = "x86")]
    use core::arch::x86::{
        __m128i, _mm_cmpeq_epi64, _mm_loadu_si128, _mm_movemask_epi8, _mm_or_si128,
        _mm_set1_epi64x, _mm_sll_epi64, _mm_srl_epi64,
    };
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m128i, _mm_cmpeq_epi64, _mm_loadu_si128, _mm_movemask_epi8, _mm_or_si128,
        _mm_set1_epi64x, _mm_sll_epi64, _mm_srl_epi64,
    };

    // -- aligned ------------------------------------------------------------

    #[target_feature(enable = "sse4.1")]
    pub(super) unsafe fn eq_words(src: &[u64], other: &[u64], len: usize) -> bool {
        let mut i = 0;
        while i + 2 <= len {
            let a = _mm_loadu_si128(src.as_ptr().add(i).cast::<__m128i>());
            let b = _mm_loadu_si128(other.as_ptr().add(i).cast::<__m128i>());
            let cmp = _mm_cmpeq_epi64(a, b);
            if _mm_movemask_epi8(cmp) as u32 != 0xFFFF {
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

    // -- unaligned ----------------------------------------------------------

    #[target_feature(enable = "sse4.1")]
    pub(super) unsafe fn eq_words_unaligned(
        src: &[u64],
        other: &[u64],
        len: usize,
        shift: usize,
    ) -> bool {
        let count_lo = _mm_set1_epi64x(shift as i64);
        let count_hi = _mm_set1_epi64x((WORD_BITS - shift) as i64);
        let mut i = 0;
        while i + 2 <= len {
            let w0 = _mm_loadu_si128(src.as_ptr().add(i).cast::<__m128i>());
            let w1 = _mm_loadu_si128(src.as_ptr().add(i + 1).cast::<__m128i>());
            let lo = _mm_srl_epi64(w0, count_lo);
            let hi = _mm_sll_epi64(w1, count_hi);
            let window = _mm_or_si128(lo, hi);
            let b = _mm_loadu_si128(other.as_ptr().add(i).cast::<__m128i>());
            let cmp = _mm_cmpeq_epi64(window, b);
            if _mm_movemask_epi8(cmp) as u32 != 0xFFFF {
                return false;
            }
            i += 2;
        }
        while i < len {
            let w0 = src[i];
            let w1 = src[i + 1];
            if ((w0 >> shift) | (w1 << (WORD_BITS - shift))) != other[i] {
                return false;
            }
            i += 1;
        }
        true
    }
}

// ===========================================================================
// NEON backend
// ===========================================================================

#[allow(unused)]
#[cfg(target_arch = "aarch64")]
mod neon {
    use crate::WORD_BITS;

    use core::arch::aarch64::{
        uint64x2_t, vceqq_u64, vdupq_n_s64, vgetq_lane_u64, vld1q_u64, vorrq_u64, vshlq_u64,
    };

    // -- aligned ------------------------------------------------------------

    #[target_feature(enable = "neon")]
    pub(super) unsafe fn eq_words(src: &[u64], other: &[u64], len: usize) -> bool {
        let mut i = 0;
        while i + 2 <= len {
            let a = vld1q_u64(src.as_ptr().add(i));
            let b = vld1q_u64(other.as_ptr().add(i));
            let cmp = vceqq_u64(a, b);
            if vgetq_lane_u64(cmp, 0) == 0 || vgetq_lane_u64(cmp, 1) == 0 {
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

    // -- unaligned ----------------------------------------------------------

    #[target_feature(enable = "neon")]
    pub(super) unsafe fn eq_words_unaligned(
        src: &[u64],
        other: &[u64],
        len: usize,
        shift: usize,
    ) -> bool {
        let neg_shift = vdupq_n_s64(-(shift as i64));
        let pos_shift = vdupq_n_s64((WORD_BITS - shift) as i64);
        let mut i = 0;
        while i + 2 <= len {
            let w0 = vld1q_u64(src.as_ptr().add(i));
            let w1 = vld1q_u64(src.as_ptr().add(i + 1));
            let lo = vshlq_u64(w0, neg_shift);
            let hi = vshlq_u64(w1, pos_shift);
            let window = vorrq_u64(lo, hi);
            let expected = vld1q_u64(other.as_ptr().add(i));
            let cmp = vceqq_u64(window, expected);
            if vgetq_lane_u64(cmp, 0) == 0 || vgetq_lane_u64(cmp, 1) == 0 {
                return false;
            }
            i += 2;
        }
        while i < len {
            let w0 = src[i];
            let w1 = src[i + 1];
            if ((w0 >> shift) | (w1 << (WORD_BITS - shift))) != other[i] {
                return false;
            }
            i += 1;
        }
        true
    }
}

#[cfg(test)]
mod tests_for_backend_equivalence;
