//! SIMD word-level unaligned (shifted-window) comparison.
//!
//! `self` has a non-zero intra-word `shift`; `other` is word-aligned.
//! Each logical word of `self` spans two source words, reconstructed as
//! `(src[i] >> shift) | (src[i+1] << (WORD_BITS - shift))`.

use core::cmp::Ordering;

use crate::traits::WordOrd;
use crate::{SMALL_WORDS, WORD_BITS};

/// Returns `Some(Ordering)` at the first differing word, or `None` when all
/// `count` shifted windows match `other`.
#[inline]
pub(super) fn cmp_unaligned_words(
    src: &[u64],
    other: &[u64],
    count: usize,
    shift: usize,
) -> Option<Ordering> {
    debug_assert!(shift > 0 && shift < WORD_BITS);

    if count < SMALL_WORDS {
        return scalar_cmp_unaligned(src, other, count, shift);
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    {
        return unsafe { avx2::cmp_unaligned(src, other, count, shift) };
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse4.1",
        not(target_feature = "avx2")
    ))]
    {
        return unsafe { sse41::cmp_unaligned(src, other, count, shift) };
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        return unsafe { neon::cmp_unaligned(src, other, count, shift) };
    }

    #[allow(unreachable_code)]
    scalar_cmp_unaligned(src, other, count, shift)
}

#[inline]
fn scalar_cmp_unaligned(
    src: &[u64],
    other: &[u64],
    count: usize,
    shift: usize,
) -> Option<Ordering> {
    for i in 0..count {
        let w0 = src[i];
        let w1 = src[i + 1];
        let window = (w0 >> shift) | (w1 << (WORD_BITS - shift));
        if window != other[i] {
            return Some(WordOrd::bitwise_cmp(window, other[i]));
        }
    }
    None
}

// ---------------------------------------------------------------------------
// AVX2 — 4 × u64 per iteration
// ---------------------------------------------------------------------------

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2 {
    use core::cmp::Ordering;

    use crate::WORD_BITS;
    use crate::traits::WordOrd;

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
    pub(super) unsafe fn cmp_unaligned(
        src: &[u64],
        other: &[u64],
        len: usize,
        shift: usize,
    ) -> Option<Ordering> {
        let count_lo = unsafe { _mm_set1_epi64x(shift as i64) };
        let count_hi = unsafe { _mm_set1_epi64x((WORD_BITS - shift) as i64) };
        let mut i = 0;
        while i + 4 <= len {
            // Load src[i..i+4] and src[i+1..i+5].
            let w0 = unsafe { _mm256_loadu_si256(src.as_ptr().add(i).cast::<__m256i>()) };
            let w1 = unsafe { _mm256_loadu_si256(src.as_ptr().add(i + 1).cast::<__m256i>()) };
            let lo = unsafe { _mm256_srl_epi64(w0, count_lo) };
            let hi = unsafe { _mm256_sll_epi64(w1, count_hi) };
            let window = unsafe { _mm256_or_si256(lo, hi) };
            let b = unsafe { _mm256_loadu_si256(other.as_ptr().add(i).cast::<__m256i>()) };
            let cmp = unsafe { _mm256_cmpeq_epi64(window, b) };
            let mask = unsafe { _mm256_movemask_pd(core::mem::transmute(cmp)) } as u32;
            if mask != 0b1111 {
                let lane = mask.trailing_ones() as usize;
                let sw = (src[i + lane] >> shift) | (src[i + lane + 1] << (WORD_BITS - shift));
                return Some(WordOrd::bitwise_cmp(sw, other[i + lane]));
            }
            i += 4;
        }
        while i < len {
            let sw = (src[i] >> shift) | (src[i + 1] << (WORD_BITS - shift));
            if sw != other[i] {
                return Some(WordOrd::bitwise_cmp(sw, other[i]));
            }
            i += 1;
        }
        None
    }
}

// ---------------------------------------------------------------------------
// SSE2 — 2 × u64 per iteration
// ---------------------------------------------------------------------------

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse41 {
    use core::cmp::Ordering;

    use crate::WORD_BITS;
    use crate::traits::WordOrd;

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

    #[target_feature(enable = "sse4.1")]
    pub(super) unsafe fn cmp_unaligned(
        src: &[u64],
        other: &[u64],
        len: usize,
        shift: usize,
    ) -> Option<Ordering> {
        let count_lo = unsafe { _mm_set1_epi64x(shift as i64) };
        let count_hi = unsafe { _mm_set1_epi64x((WORD_BITS - shift) as i64) };
        let mut i = 0;
        while i + 2 <= len {
            // Load src[i..i+2] and src[i+1..i+3].
            let w0 = unsafe { _mm_loadu_si128(src.as_ptr().add(i).cast::<__m128i>()) };
            let w1 = unsafe { _mm_loadu_si128(src.as_ptr().add(i + 1).cast::<__m128i>()) };
            let lo = unsafe { _mm_srl_epi64(w0, count_lo) };
            let hi = unsafe { _mm_sll_epi64(w1, count_hi) };
            let window = unsafe { _mm_or_si128(lo, hi) };
            let b = unsafe { _mm_loadu_si128(other.as_ptr().add(i).cast::<__m128i>()) };
            let cmp = unsafe { _mm_cmpeq_epi64(window, b) };
            let mask = unsafe { _mm_movemask_epi8(cmp) } as u32;
            if mask != 0xFFFF {
                let lane = mask.trailing_ones() as usize / 8;
                let sw = (src[i + lane] >> shift) | (src[i + lane + 1] << (WORD_BITS - shift));
                return Some(WordOrd::bitwise_cmp(sw, other[i + lane]));
            }
            i += 2;
        }
        while i < len {
            let sw = (src[i] >> shift) | (src[i + 1] << (WORD_BITS - shift));
            if sw != other[i] {
                return Some(WordOrd::bitwise_cmp(sw, other[i]));
            }
            i += 1;
        }
        None
    }
}

// ---------------------------------------------------------------------------
// NEON — 2 × u64 per iteration
// ---------------------------------------------------------------------------

#[allow(unused)]
#[cfg(target_arch = "aarch64")]
mod neon {
    use core::cmp::Ordering;

    use crate::WORD_BITS;
    use crate::traits::WordOrd;

    use core::arch::aarch64::{
        vceqq_u64, vdupq_n_s64, vgetq_lane_u64, vld1q_u64, vorrq_u64, vshlq_u64,
    };

    #[target_feature(enable = "neon")]
    pub(super) unsafe fn cmp_unaligned(
        src: &[u64],
        other: &[u64],
        len: usize,
        shift: usize,
    ) -> Option<Ordering> {
        let neg_shift = unsafe { vdupq_n_s64(-(shift as i64)) };
        let pos_shift = unsafe { vdupq_n_s64((WORD_BITS - shift) as i64) };
        let mut i = 0;
        while i + 2 <= len {
            let w0 = unsafe { vld1q_u64(src.as_ptr().add(i)) };
            let w1 = unsafe { vld1q_u64(src.as_ptr().add(i + 1)) };
            let lo = unsafe { vshlq_u64(w0, neg_shift) };
            let hi = unsafe { vshlq_u64(w1, pos_shift) };
            let window = unsafe { vorrq_u64(lo, hi) };
            let expected = unsafe { vld1q_u64(other.as_ptr().add(i)) };
            let cmp = unsafe { vceqq_u64(window, expected) };
            if unsafe { vgetq_lane_u64(cmp, 0) } == 0 {
                let sw = (src[i] >> shift) | (src[i + 1] << (WORD_BITS - shift));
                return Some(WordOrd::bitwise_cmp(sw, other[i]));
            }
            if unsafe { vgetq_lane_u64(cmp, 1) } == 0 {
                let sw = (src[i + 1] >> shift) | (src[i + 2] << (WORD_BITS - shift));
                return Some(WordOrd::bitwise_cmp(sw, other[i + 1]));
            }
            i += 2;
        }
        while i < len {
            let sw = (src[i] >> shift) | (src[i + 1] << (WORD_BITS - shift));
            if sw != other[i] {
                return Some(WordOrd::bitwise_cmp(sw, other[i]));
            }
            i += 1;
        }
        None
    }
}

#[cfg(test)]
mod tests_for_backend_equivalence;
