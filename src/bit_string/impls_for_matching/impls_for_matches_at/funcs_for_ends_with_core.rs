//! SIMD word-level equality for `ends_with`.
//!
//! When the suffix starts at a word-aligned position (`shift == 0`) this
//! delegates to [`super::funcs_for_starts_with_core::starts_with_words`].
//! Otherwise it computes shifted 64-bit windows via SIMD.

use crate::SMALL_WORDS;
use crate::WORD_BITS;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[inline]
pub(super) fn ends_with_words(sw: &[u64], pw: &[u64], full_words: usize, shift: usize) -> bool {
    if shift == 0 {
        return super::funcs_for_starts_with_core::starts_with_words(sw, pw, full_words);
    }

    if full_words < SMALL_WORDS {
        for i in 0..full_words {
            let w0 = sw[i];
            let w1 = sw[i + 1];
            if ((w0 >> shift) | (w1 << (WORD_BITS - shift))) != pw[i] {
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
        return unsafe { avx2::eq_words_unaligned(sw, pw, full_words, shift) };
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse2",
        not(target_feature = "avx2")
    ))]
    {
        return unsafe { sse2::eq_words_unaligned(sw, pw, full_words, shift) };
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        return unsafe { neon::eq_words_unaligned(sw, pw, full_words, shift) };
    }

    #[allow(unused)]
    {
        for i in 0..full_words {
            let w0 = sw[i];
            let w1 = sw[i + 1];
            if ((w0 >> shift) | (w1 << (WORD_BITS - shift))) != pw[i] {
                return false;
            }
        }
        true
    }
}

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

    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn eq_words_unaligned(
        sw: &[u64],
        pw: &[u64],
        len: usize,
        shift: usize,
    ) -> bool {
        let count_lo = unsafe { _mm_set1_epi64x(shift as i64) };
        let count_hi = unsafe { _mm_set1_epi64x((WORD_BITS - shift) as i64) };
        let mut i = 0;
        while i + 4 <= len {
            let w0 = unsafe { _mm256_loadu_si256(sw.as_ptr().add(i).cast::<__m256i>()) };
            let w1 = unsafe { _mm256_loadu_si256(sw.as_ptr().add(i + 1).cast::<__m256i>()) };
            let lo = unsafe { _mm256_srl_epi64(w0, count_lo) };
            let hi = unsafe { _mm256_sll_epi64(w1, count_hi) };
            let window = unsafe { _mm256_or_si256(lo, hi) };
            let b = unsafe { _mm256_loadu_si256(pw.as_ptr().add(i).cast::<__m256i>()) };
            let cmp = unsafe { _mm256_cmpeq_epi64(window, b) };
            if unsafe { _mm256_movemask_pd(core::mem::transmute(cmp)) } as u32 != 0b1111 {
                return false;
            }
            i += 4;
        }
        while i < len {
            let w0 = sw[i];
            let w1 = sw[i + 1];
            if ((w0 >> shift) | (w1 << (WORD_BITS - shift))) != pw[i] {
                return false;
            }
            i += 1;
        }
        true
    }
}

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse2 {
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

    #[target_feature(enable = "sse2")]
    pub(super) unsafe fn eq_words_unaligned(
        sw: &[u64],
        pw: &[u64],
        len: usize,
        shift: usize,
    ) -> bool {
        let count_lo = unsafe { _mm_set1_epi64x(shift as i64) };
        let count_hi = unsafe { _mm_set1_epi64x((WORD_BITS - shift) as i64) };
        let mut i = 0;
        while i + 2 <= len {
            let w0 = unsafe { _mm_loadu_si128(sw.as_ptr().add(i).cast::<__m128i>()) };
            let w1 = unsafe { _mm_loadu_si128(sw.as_ptr().add(i + 1).cast::<__m128i>()) };
            let lo = unsafe { _mm_srl_epi64(w0, count_lo) };
            let hi = unsafe { _mm_sll_epi64(w1, count_hi) };
            let window = unsafe { _mm_or_si128(lo, hi) };
            let b = unsafe { _mm_loadu_si128(pw.as_ptr().add(i).cast::<__m128i>()) };
            let cmp = unsafe { _mm_cmpeq_epi64(window, b) };
            if unsafe { _mm_movemask_epi8(cmp) } as u32 != 0xFFFF {
                return false;
            }
            i += 2;
        }
        while i < len {
            let w0 = sw[i];
            let w1 = sw[i + 1];
            if ((w0 >> shift) | (w1 << (WORD_BITS - shift))) != pw[i] {
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
    use crate::WORD_BITS;
    use core::arch::aarch64::{
        vceqq_u64, vdupq_n_s64, vgetq_lane_u64, vld1q_u64, vorrq_u64, vshlq_u64,
    };

    #[target_feature(enable = "neon")]
    pub(super) unsafe fn eq_words_unaligned(
        sw: &[u64],
        pw: &[u64],
        len: usize,
        shift: usize,
    ) -> bool {
        // SAFETY: `shift` is in [1, WORD_BITS); the caller guarantees this
        // via the `shift == 0` fast-path in the entry point.
        // Both shift vectors fit in i64:
        //   shift ∈ [1, 63]  →  -shift ∈ [-63, -1]
        //   WORD_BITS - shift ∈ [1, 63]
        let neg_shift = unsafe { vdupq_n_s64(-(shift as i64)) };
        let pos_shift = unsafe { vdupq_n_s64((WORD_BITS - shift) as i64) };

        // Process 2 lanes (128 bits) per iteration.
        let mut i = 0;
        while i + 2 <= len {
            // Load [sw[i], sw[i+1]] and [sw[i+1], sw[i+2]].
            let w0 = unsafe { vld1q_u64(sw.as_ptr().add(i)) };
            let w1 = unsafe { vld1q_u64(sw.as_ptr().add(i + 1)) };

            // Build the shifted 64-bit window for each lane:
            //   window[k] = (sw[i+k] >> shift) | (sw[i+k+1] << (64 - shift))
            // vshlq_u64 with a negative shift amount performs a logical right shift.
            let lo = unsafe { vshlq_u64(w0, neg_shift) };
            let hi = unsafe { vshlq_u64(w1, pos_shift) };
            let window = unsafe { vorrq_u64(lo, hi) };

            let expected = unsafe { vld1q_u64(pw.as_ptr().add(i)) };
            let cmp = unsafe { vceqq_u64(window, expected) };

            // Each lane is all-ones on equality → vgetq_lane_u64 returns u64::MAX.
            if unsafe { vgetq_lane_u64(cmp, 0) } == 0 || unsafe { vgetq_lane_u64(cmp, 1) } == 0 {
                return false;
            }

            i += 2;
        }

        // Scalar tail for the last word (when len is odd).
        while i < len {
            let w0 = sw[i];
            let w1 = sw[i + 1];
            if ((w0 >> shift) | (w1 << (WORD_BITS - shift))) != pw[i] {
                return false;
            }
            i += 1;
        }

        true
    }
}

#[cfg(test)]
mod tests_for_backend_equivalence;
