//! SIMD word-level unaligned (shifted-window) equality.
//!
//! Computes shifted 64-bit windows and compares each against the
//! corresponding word in `other`.  The caller must ensure `shift != 0`;
//! the `shift == 0` fast path is handled by the trait impl.

use crate::SMALL_WORDS;
use crate::WORD_BITS;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[inline]
pub(super) fn eq_words_unaligned(src: &[u64], other: &[u64], count: usize, shift: usize) -> bool {
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
        // SAFETY: `src`/`other` are valid for `count+1` words (caller ensures extra word for shifting). Backend feature is guaranteed by compile-time `#[cfg]` gate.
        return unsafe { avx2::eq_words_unaligned(src, other, count, shift) };
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse4.1",
        not(target_feature = "avx2")
    ))]
    {
        // SAFETY: `src`/`other` are valid for `count+1` words (caller ensures extra word for shifting). Backend feature is guaranteed by compile-time `#[cfg]` gate.
        return unsafe { sse41::eq_words_unaligned(src, other, count, shift) };
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        // SAFETY: `src`/`other` are valid for `count+1` words (caller ensures extra word for shifting). Backend feature is enabled by the `#[cfg]` gate on this block.
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
        src: &[u64],
        other: &[u64],
        len: usize,
        shift: usize,
    ) -> bool {
        // SAFETY: `shift` and `WORD_BITS - shift` fit in i64; AVX2 is enabled by `#[target_feature]`.
        let count_lo = unsafe { _mm_set1_epi64x(shift as i64) };
        // SAFETY: same as above
        let count_hi = unsafe { _mm_set1_epi64x((WORD_BITS - shift) as i64) };
        let mut i = 0;
        while i + 4 <= len {
            // SAFETY: `#[target_feature(enable = "avx2")]` ensures AVX2 is enabled.
            // Pointers `src` and `other` are valid for `len+1` elements (caller guarantees extra word for window shift).
            let w0 = unsafe { _mm256_loadu_si256(src.as_ptr().add(i).cast::<__m256i>()) };
            // SAFETY: same as above; load from `src[i+1]`.
            let w1 = unsafe { _mm256_loadu_si256(src.as_ptr().add(i + 1).cast::<__m256i>()) };
            // SAFETY: `_mm256_srl_epi64`, `_mm256_sll_epi64`, `_mm256_or_si256`, `_mm256_cmpeq_epi64`, and `_mm256_movemask_pd` are pure register operations; AVX2 is enabled by `#[target_feature]`.
            let lo = unsafe { _mm256_srl_epi64(w0, count_lo) };
            // SAFETY: same as above
            let hi = unsafe { _mm256_sll_epi64(w1, count_hi) };
            // SAFETY: same as above
            let window = unsafe { _mm256_or_si256(lo, hi) };
            // SAFETY: AVX2 is enabled; pointer `other` is valid for `len` elements.
            let b = unsafe { _mm256_loadu_si256(other.as_ptr().add(i).cast::<__m256i>()) };
            // SAFETY: `_mm256_cmpeq_epi64` is a pure register operation; AVX2 is enabled by `#[target_feature]`.
            let cmp = unsafe { _mm256_cmpeq_epi64(window, b) };
            // SAFETY: same as above
            if unsafe { _mm256_movemask_pd(core::mem::transmute(cmp)) } as u32 != 0b1111 {
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

    #[target_feature(enable = "sse4.1")]
    pub(super) unsafe fn eq_words_unaligned(
        src: &[u64],
        other: &[u64],
        len: usize,
        shift: usize,
    ) -> bool {
        // SAFETY: `shift` and `WORD_BITS - shift` fit in i64; SSE4.1 is enabled by `#[target_feature]`.
        let count_lo = unsafe { _mm_set1_epi64x(shift as i64) };
        // SAFETY: same as above
        let count_hi = unsafe { _mm_set1_epi64x((WORD_BITS - shift) as i64) };
        let mut i = 0;
        while i + 2 <= len {
            // SAFETY: `#[target_feature(enable = "sse4.1")]` ensures SSE4.1 is enabled.
            // Pointers `src` and `other` are valid for `len+1` and `len` elements respectively (caller guarantees).
            let w0 = unsafe { _mm_loadu_si128(src.as_ptr().add(i).cast::<__m128i>()) };
            // SAFETY: same as above; load from `src[i+1]`.
            let w1 = unsafe { _mm_loadu_si128(src.as_ptr().add(i + 1).cast::<__m128i>()) };
            // SAFETY: `_mm_srl_epi64`, `_mm_sll_epi64`, `_mm_or_si128`, `_mm_cmpeq_epi64`, and `_mm_movemask_epi8` are pure register operations; SSE4.1 is enabled by `#[target_feature]`.
            let lo = unsafe { _mm_srl_epi64(w0, count_lo) };
            // SAFETY: same as above
            let hi = unsafe { _mm_sll_epi64(w1, count_hi) };
            // SAFETY: same as above
            let window = unsafe { _mm_or_si128(lo, hi) };
            // SAFETY: SSE4.1 is enabled; pointer `other` is valid for `len` elements.
            let b = unsafe { _mm_loadu_si128(other.as_ptr().add(i).cast::<__m128i>()) };
            // SAFETY: `_mm_cmpeq_epi64` is a pure register operation; SSE4.1 is enabled by `#[target_feature]`.
            let cmp = unsafe { _mm_cmpeq_epi64(window, b) };
            // SAFETY: same as above
            if unsafe { _mm_movemask_epi8(cmp) } as u32 != 0xFFFF {
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

#[allow(unused)]
#[cfg(target_arch = "aarch64")]
mod neon {
    use crate::WORD_BITS;
    use core::arch::aarch64::{
        vceqq_u64, vdupq_n_s64, vgetq_lane_u64, vld1q_u64, vorrq_u64, vshlq_u64,
    };

    #[target_feature(enable = "neon")]
    pub(super) unsafe fn eq_words_unaligned(
        src: &[u64],
        other: &[u64],
        len: usize,
        shift: usize,
    ) -> bool {
        // SAFETY: `shift` is in [1, WORD_BITS); the caller guarantees this
        // via the `shift == 0` fast-path in the entry point.
        // Both shift vectors fit in i64:
        //   shift ∈ [1, 63]  →  -shift ∈ [-63, -1]
        //   WORD_BITS - shift ∈ [1, 63]
        let neg_shift = unsafe { vdupq_n_s64(-(shift as i64)) };
        // SAFETY: same as above
        let pos_shift = unsafe { vdupq_n_s64((WORD_BITS - shift) as i64) };

        // Process 2 lanes (128 bits) per iteration.
        let mut i = 0;
        while i + 2 <= len {
            // SAFETY: `#[target_feature(enable = "neon")]` ensures NEON is enabled.
            // Pointers `src` and `other` are valid for `len+1` and `len` elements respectively (caller guarantees extra word for window shift).
            // Load [src[i], src[i+1]] and [src[i+1], src[i+2]].
            let w0 = unsafe { vld1q_u64(src.as_ptr().add(i)) };
            // SAFETY: same as above; load from `src[i+1]`.
            let w1 = unsafe { vld1q_u64(src.as_ptr().add(i + 1)) };

            // Build the shifted 64-bit window for each lane:
            //   window[k] = (src[i+k] >> shift) | (src[i+k+1] << (64 - shift))
            // vshlq_u64 with a negative shift amount performs a logical right shift.
            // SAFETY: `vshlq_u64`, `vorrq_u64`, `vceqq_u64`, and `vgetq_lane_u64` are pure register operations; NEON is enabled by `#[target_feature]`.
            let lo = unsafe { vshlq_u64(w0, neg_shift) };
            // SAFETY: same as above
            let hi = unsafe { vshlq_u64(w1, pos_shift) };
            // SAFETY: same as above
            let window = unsafe { vorrq_u64(lo, hi) };

            // SAFETY: NEON is enabled; pointer `other` is valid for `len` elements.
            let expected = unsafe { vld1q_u64(other.as_ptr().add(i)) };
            // SAFETY: `vceqq_u64` is a pure register operation; NEON is enabled by `#[target_feature]`.
            let cmp = unsafe { vceqq_u64(window, expected) };

            // Each lane is all-ones on equality → vgetq_lane_u64 returns u64::MAX.
            // SAFETY: `vgetq_lane_u64` is a pure register operation; NEON is enabled by `#[target_feature]`.
            if unsafe { vgetq_lane_u64(cmp, 0) } == 0 || unsafe { vgetq_lane_u64(cmp, 1) } == 0 {
                return false;
            }

            i += 2;
        }

        // Scalar tail for the last word (when len is odd).
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
