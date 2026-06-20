//! SIMD word-level equality for `starts_with` and `ends_with`.

use crate::SMALL_WORDS;
use crate::WORD_BITS;

// ---------------------------------------------------------------------------
// starts_with — position 0, always word-aligned
// ---------------------------------------------------------------------------

#[inline]
pub(super) fn starts_with_words(sw: &[u64], pw: &[u64], full_words: usize) -> bool {
    if full_words < SMALL_WORDS {
        for i in 0..full_words {
            if sw[i] != pw[i] {
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
        return unsafe { avx2::eq_words(sw, pw, full_words) };
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse2",
        not(target_feature = "avx2")
    ))]
    {
        return unsafe { sse2::eq_words(sw, pw, full_words) };
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        return unsafe { neon::eq_words(sw, pw, full_words) };
    }

    #[allow(unused)]
    {
        for i in 0..full_words {
            if sw[i] != pw[i] {
                return false;
            }
        }
        true
    }
}

// ---------------------------------------------------------------------------
// ends_with — position may be unaligned (shift != 0)
// ---------------------------------------------------------------------------

#[inline]
pub(super) fn ends_with_words(sw: &[u64], pw: &[u64], full_words: usize, shift: usize) -> bool {
    if shift == 0 {
        return starts_with_words(sw, pw, full_words);
    }

    if full_words < SMALL_WORDS {
        for i in 0..full_words {
            let w0 = sw[i];
            let w1 = sw[i + 1];
            let window = (w0 >> shift) | (w1 << (WORD_BITS - shift));
            if window != pw[i] {
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

// ---------------------------------------------------------------------------
// AVX2
// ---------------------------------------------------------------------------

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2 {
    use crate::WORD_BITS;

    #[cfg(target_arch = "x86")]
    use core::arch::x86::{
        __m128i, __m256i, _mm_set1_epi64x, _mm256_cmpeq_epi64, _mm256_loadu_si256,
        _mm256_movemask_pd, _mm256_or_si256, _mm256_set1_epi64x, _mm256_sll_epi64,
        _mm256_srl_epi64,
    };
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m128i, __m256i, _mm_set1_epi64x, _mm256_cmpeq_epi64, _mm256_loadu_si256,
        _mm256_movemask_pd, _mm256_or_si256, _mm256_set1_epi64x, _mm256_sll_epi64,
        _mm256_srl_epi64,
    };

    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn eq_words(sw: &[u64], pw: &[u64], len: usize) -> bool {
        let mut i = 0;
        while i + 4 <= len {
            let a = unsafe { _mm256_loadu_si256(sw.as_ptr().add(i).cast::<__m256i>()) };
            let b = unsafe { _mm256_loadu_si256(pw.as_ptr().add(i).cast::<__m256i>()) };
            let cmp = unsafe { _mm256_cmpeq_epi64(a, b) };
            if unsafe { _mm256_movemask_pd(core::mem::transmute(cmp)) } as u32 != 0b1111 {
                return false;
            }
            i += 4;
        }
        while i < len {
            if sw[i] != pw[i] {
                return false;
            }
            i += 1;
        }
        true
    }

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

// ---------------------------------------------------------------------------
// SSE2
// ---------------------------------------------------------------------------

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
    pub(super) unsafe fn eq_words(sw: &[u64], pw: &[u64], len: usize) -> bool {
        let mut i = 0;
        while i + 2 <= len {
            let a = unsafe { _mm_loadu_si128(sw.as_ptr().add(i).cast::<__m128i>()) };
            let b = unsafe { _mm_loadu_si128(pw.as_ptr().add(i).cast::<__m128i>()) };
            let cmp = unsafe { _mm_cmpeq_epi64(a, b) };
            if unsafe { _mm_movemask_epi8(cmp) } as u32 != 0xFFFF {
                return false;
            }
            i += 2;
        }
        while i < len {
            if sw[i] != pw[i] {
                return false;
            }
            i += 1;
        }
        true
    }

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

// ---------------------------------------------------------------------------
// NEON
// ---------------------------------------------------------------------------

#[allow(unused)]
#[cfg(target_arch = "aarch64")]
mod neon {
    use crate::WORD_BITS;

    use core::arch::aarch64::{uint64x2_t, vceqq_u64, vgetq_lane_u64, vld1q_u64};

    #[target_feature(enable = "neon")]
    pub(super) unsafe fn eq_words(sw: &[u64], pw: &[u64], len: usize) -> bool {
        let mut i = 0;
        while i + 2 <= len {
            let a = unsafe { vld1q_u64(sw.as_ptr().add(i)) };
            let b = unsafe { vld1q_u64(pw.as_ptr().add(i)) };
            let cmp = unsafe { vceqq_u64(a, b) };
            if unsafe { vgetq_lane_u64(cmp, 0) } == 0 || unsafe { vgetq_lane_u64(cmp, 1) } == 0 {
                return false;
            }
            i += 2;
        }
        while i < len {
            if sw[i] != pw[i] {
                return false;
            }
            i += 1;
        }
        true
    }

    #[target_feature(enable = "neon")]
    pub(super) unsafe fn eq_words_unaligned(
        sw: &[u64],
        pw: &[u64],
        len: usize,
        shift: usize,
    ) -> bool {
        let mut i = 0;
        while i + 2 <= len {
            // NEON doesn't have convenient per-lane variable shifts — fall back to scalar per-pair.
            for k in 0..2 {
                let w0 = sw[i + k];
                let w1 = sw[i + k + 1];
                if ((w0 >> shift) | (w1 << (WORD_BITS - shift))) != pw[i + k] {
                    return false;
                }
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
