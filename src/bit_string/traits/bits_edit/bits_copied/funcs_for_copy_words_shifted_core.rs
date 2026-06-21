//! SIMD shifted-window copy.
//!
//! Copies `count` shifted 64-bit windows from `src` to `dst`:
//!
//! ```text
//! dst[i] = (src[i] >> shift) | (src[i + 1] << (WORD_BITS - shift))
//! ```
//!
//! The destination is always word-aligned.  Short inputs fall back to scalar.

use crate::SMALL_WORDS;
use crate::WORD_BITS;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Copy `count` shifted 64-bit windows from `src` into `dst`.
///
/// The caller guarantees `dst` has room for `count` words, `src` has at
/// least `count + 1` words, and `shift ∈ [1, WORD_BITS)`.
#[inline]
pub(super) fn copy_words_shifted(dst: &mut [u64], src: &[u64], count: usize, shift: usize) {
    debug_assert!(shift > 0 && shift < WORD_BITS);

    if count < SMALL_WORDS {
        for i in 0..count {
            dst[i] = (src[i] >> shift) | (src[i + 1] << (WORD_BITS - shift));
        }
        return;
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    {
        unsafe { avx2::copy_words_shifted(dst, src, count, shift) };
        return;
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse2",
        not(target_feature = "avx2")
    ))]
    {
        unsafe { sse2::copy_words_shifted(dst, src, count, shift) };
        return;
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        unsafe { neon::copy_words_shifted(dst, src, count, shift) };
        return;
    }

    #[allow(unused)]
    {
        for i in 0..count {
            dst[i] = (src[i] >> shift) | (src[i + 1] << (WORD_BITS - shift));
        }
    }
}

// ---------------------------------------------------------------------------
// AVX2 — 4 words per iteration
// ---------------------------------------------------------------------------

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2 {
    use crate::WORD_BITS;

    #[cfg(target_arch = "x86")]
    use core::arch::x86::{
        __m128i, __m256i, _mm_set1_epi64x, _mm256_loadu_si256, _mm256_or_si256, _mm256_sll_epi64,
        _mm256_srl_epi64, _mm256_storeu_si256,
    };
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m128i, __m256i, _mm_set1_epi64x, _mm256_loadu_si256, _mm256_or_si256, _mm256_sll_epi64,
        _mm256_srl_epi64, _mm256_storeu_si256,
    };

    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn copy_words_shifted(
        dst: &mut [u64],
        src: &[u64],
        len: usize,
        shift: usize,
    ) {
        let count_lo = unsafe { _mm_set1_epi64x(shift as i64) };
        let count_hi = unsafe { _mm_set1_epi64x((WORD_BITS - shift) as i64) };
        let mut i = 0;
        while i + 4 <= len {
            let w0 = unsafe { _mm256_loadu_si256(src.as_ptr().add(i).cast::<__m256i>()) };
            let w1 = unsafe { _mm256_loadu_si256(src.as_ptr().add(i + 1).cast::<__m256i>()) };
            let lo = unsafe { _mm256_srl_epi64(w0, count_lo) };
            let hi = unsafe { _mm256_sll_epi64(w1, count_hi) };
            let window = unsafe { _mm256_or_si256(lo, hi) };
            unsafe { _mm256_storeu_si256(dst.as_mut_ptr().add(i).cast::<__m256i>(), window) };
            i += 4;
        }
        while i < len {
            dst[i] = (src[i] >> shift) | (src[i + 1] << (WORD_BITS - shift));
            i += 1;
        }
    }
}

// ---------------------------------------------------------------------------
// SSE2 — 2 words per iteration
// ---------------------------------------------------------------------------

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse2 {
    use crate::WORD_BITS;

    #[cfg(target_arch = "x86")]
    use core::arch::x86::{
        __m128i, _mm_loadu_si128, _mm_or_si128, _mm_set1_epi64x, _mm_sll_epi64, _mm_srl_epi64,
        _mm_storeu_si128,
    };
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m128i, _mm_loadu_si128, _mm_or_si128, _mm_set1_epi64x, _mm_sll_epi64, _mm_srl_epi64,
        _mm_storeu_si128,
    };

    #[target_feature(enable = "sse2")]
    pub(super) unsafe fn copy_words_shifted(
        dst: &mut [u64],
        src: &[u64],
        len: usize,
        shift: usize,
    ) {
        let count_lo = unsafe { _mm_set1_epi64x(shift as i64) };
        let count_hi = unsafe { _mm_set1_epi64x((WORD_BITS - shift) as i64) };
        let mut i = 0;
        while i + 2 <= len {
            let w0 = unsafe { _mm_loadu_si128(src.as_ptr().add(i).cast::<__m128i>()) };
            let w1 = unsafe { _mm_loadu_si128(src.as_ptr().add(i + 1).cast::<__m128i>()) };
            let lo = unsafe { _mm_srl_epi64(w0, count_lo) };
            let hi = unsafe { _mm_sll_epi64(w1, count_hi) };
            let window = unsafe { _mm_or_si128(lo, hi) };
            unsafe { _mm_storeu_si128(dst.as_mut_ptr().add(i).cast::<__m128i>(), window) };
            i += 2;
        }
        while i < len {
            dst[i] = (src[i] >> shift) | (src[i + 1] << (WORD_BITS - shift));
            i += 1;
        }
    }
}

// ---------------------------------------------------------------------------
// NEON — 2 words per iteration
// ---------------------------------------------------------------------------

#[allow(unused)]
#[cfg(target_arch = "aarch64")]
mod neon {
    use crate::WORD_BITS;
    use core::arch::aarch64::{vdupq_n_s64, vld1q_u64, vorrq_u64, vshlq_u64, vst1q_u64};

    #[target_feature(enable = "neon")]
    pub(super) unsafe fn copy_words_shifted(
        dst: &mut [u64],
        src: &[u64],
        len: usize,
        shift: usize,
    ) {
        let neg_shift = unsafe { vdupq_n_s64(-(shift as i64)) };
        let pos_shift = unsafe { vdupq_n_s64((WORD_BITS - shift) as i64) };

        let mut i = 0;
        while i + 2 <= len {
            let w0 = unsafe { vld1q_u64(src.as_ptr().add(i)) };
            let w1 = unsafe { vld1q_u64(src.as_ptr().add(i + 1)) };
            let lo = unsafe { vshlq_u64(w0, neg_shift) };
            let hi = unsafe { vshlq_u64(w1, pos_shift) };
            let window = unsafe { vorrq_u64(lo, hi) };
            unsafe { vst1q_u64(dst.as_mut_ptr().add(i), window) };
            i += 2;
        }
        while i < len {
            dst[i] = (src[i] >> shift) | (src[i + 1] << (WORD_BITS - shift));
            i += 1;
        }
    }
}

#[cfg(test)]
mod tests_for_backend_equivalence;
