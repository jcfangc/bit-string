//! SIMD first-word pre-filter for `contains`, `find`, and `rfind`.
//!
//! Returns `Some(pos)` for the first candidate whose 64-bit window
//! matches the needle's first word AND `verify(pos)` succeeds.
//!
//! Uses **shift-outer, word-inner** ordering, processing LANES haystack
//! words in parallel per shift.  This ordering does **not** guarantee
//! the returned position is the earliest match — `find` must use a
//! binary-search driver or a word-outer scan for correct ordering.

use crate::SMALL_WORDS;
use crate::WORD_BITS;
use crate::funcs_for_bits::low_mask;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Returns `Some(pos)` if any 64-bit window in `haystack[0..word_limit]`
/// matches the first word of `needle_words` AND `verify(pos)` succeeds.
///
/// Scans positions `pos ∈ [0, haystack_bit_len - needle_bit_len]` using
/// **shift-outer, word-inner** ordering — does **not** guarantee the
/// returned position is the earliest match.
#[inline]
pub(super) fn find_first_candidate<F>(
    haystack: &[u64],
    haystack_bit_len: usize,
    needle_words: &[u64],
    needle_bit_len: usize,
    verify: &mut F,
) -> Option<usize>
where
    F: FnMut(usize) -> bool,
{
    let needle_first = needle_words[0];
    let needle_mask = low_mask(needle_bit_len.min(WORD_BITS));
    let last_start = haystack_bit_len - needle_bit_len;
    let max_word = last_start / WORD_BITS;
    let word_limit = (max_word + 1).min(haystack.len());

    if haystack.len() < SMALL_WORDS {
        return scalar(
            haystack,
            needle_first,
            needle_mask,
            last_start,
            word_limit,
            verify,
        );
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    {
        unsafe {
            return avx2::find_first(
                haystack,
                needle_first,
                needle_mask,
                last_start,
                word_limit,
                verify,
            );
        }
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse2",
        not(target_feature = "avx2")
    ))]
    {
        unsafe {
            return sse2::find_first(
                haystack,
                needle_first,
                needle_mask,
                last_start,
                word_limit,
                verify,
            );
        }
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        unsafe {
            return neon::find_first(
                haystack,
                needle_first,
                needle_mask,
                last_start,
                word_limit,
                verify,
            );
        }
    }

    #[allow(unused)]
    scalar(
        haystack,
        needle_first,
        needle_mask,
        last_start,
        word_limit,
        verify,
    )
}

// ---------------------------------------------------------------------------
// Scalar fallback
// ---------------------------------------------------------------------------

/// Word-by-word scan: for each shift, check every word pair in
/// `[0, word_limit)` for a matching window.
fn scalar<F>(
    haystack: &[u64],
    needle_first: u64,
    needle_mask: u64,
    last_start: usize,
    word_limit: usize,
    verify: &mut F,
) -> Option<usize>
where
    F: FnMut(usize) -> bool,
{
    for shift in 0..WORD_BITS {
        for i in 0..word_limit {
            let pos = i * WORD_BITS + shift;
            if pos > last_start {
                break;
            }
            let window = if shift == 0 {
                haystack[i]
            } else {
                let w0 = haystack[i];
                let w1 = haystack.get(i + 1).copied().unwrap_or(0);
                (w0 >> shift) | (w1 << (WORD_BITS - shift))
            };
            if (window & needle_mask) == needle_first && verify(pos) {
                return Some(pos);
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// SSE2 — 2 haystack words per iteration
// ---------------------------------------------------------------------------

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse2 {
    use super::*;

    #[cfg(target_arch = "x86")]
    use core::arch::x86::{
        __m128i, _mm_and_si128, _mm_cmpeq_epi64, _mm_loadu_si128, _mm_movemask_epi8, _mm_or_si128,
        _mm_set1_epi64x, _mm_sll_epi64, _mm_srl_epi64,
    };
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m128i, _mm_and_si128, _mm_cmpeq_epi64, _mm_loadu_si128, _mm_movemask_epi8, _mm_or_si128,
        _mm_set1_epi64x, _mm_sll_epi64, _mm_srl_epi64,
    };

    const LANES: usize = 2;

    /// SSE2 backend: loads 2 consecutive words, computes a sliding
    /// window for the current shift, and compares against the broadcast
    /// needle word.  `movemask` extracts match lanes.
    #[target_feature(enable = "sse2")]
    pub(super) unsafe fn find_first<F>(
        haystack: &[u64],
        needle_first: u64,
        needle_mask: u64,
        last_start: usize,
        word_limit: usize,
        verify: &mut F,
    ) -> Option<usize>
    where
        F: FnMut(usize) -> bool,
    {
        let needle = unsafe { _mm_set1_epi64x(needle_first as i64) };
        let mask = unsafe { _mm_set1_epi64x(needle_mask as i64) };

        for shift in 0..WORD_BITS {
            let mut i = 0;
            while i + LANES <= word_limit {
                if i * WORD_BITS + shift > last_start {
                    break;
                }

                let window = if shift == 0 {
                    unsafe { _mm_loadu_si128(haystack.as_ptr().add(i).cast::<__m128i>()) }
                } else {
                    let src = haystack.as_ptr();
                    let w01 = unsafe { _mm_loadu_si128(src.add(i).cast::<__m128i>()) };
                    let w12 = unsafe { _mm_loadu_si128(src.add(i + 1).cast::<__m128i>()) };
                    let count_lo = unsafe { _mm_set1_epi64x(shift as i64) };
                    let count_hi = unsafe { _mm_set1_epi64x((WORD_BITS - shift) as i64) };
                    let lo = unsafe { _mm_srl_epi64(w01, count_lo) };
                    let hi = unsafe { _mm_sll_epi64(w12, count_hi) };
                    unsafe { _mm_or_si128(lo, hi) }
                };

                let masked = unsafe { _mm_and_si128(window, mask) };
                let cmp = unsafe { _mm_cmpeq_epi64(masked, needle) };
                let hits = unsafe { _mm_movemask_epi8(cmp) } as u32;

                if hits & 0xff != 0 {
                    let pos = i * WORD_BITS + shift;
                    if pos <= last_start && verify(pos) {
                        return Some(pos);
                    }
                }
                if hits & 0xff00 != 0 {
                    let pos = (i + 1) * WORD_BITS + shift;
                    if pos <= last_start && verify(pos) {
                        return Some(pos);
                    }
                }

                i += LANES;
            }

            for j in i..word_limit {
                let pos = j * WORD_BITS + shift;
                if pos > last_start {
                    break;
                }
                let window = if shift == 0 {
                    haystack[j]
                } else {
                    let w0 = haystack[j];
                    let w1 = haystack.get(j + 1).copied().unwrap_or(0);
                    (w0 >> shift) | (w1 << (WORD_BITS - shift))
                };
                if (window & needle_mask) == needle_first && verify(pos) {
                    return Some(pos);
                }
            }
        }

        None
    }
}

// ---------------------------------------------------------------------------
// AVX2 — 4 haystack words per iteration
// ---------------------------------------------------------------------------

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2 {
    use super::*;

    #[cfg(target_arch = "x86")]
    use core::arch::x86::{
        __m128i, __m256i, _mm_set1_epi64x, _mm256_and_si256, _mm256_cmpeq_epi64,
        _mm256_loadu_si256, _mm256_movemask_pd, _mm256_or_si256, _mm256_set1_epi64x,
        _mm256_sll_epi64, _mm256_srl_epi64,
    };
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m128i, __m256i, _mm_set1_epi64x, _mm256_and_si256, _mm256_cmpeq_epi64,
        _mm256_loadu_si256, _mm256_movemask_pd, _mm256_or_si256, _mm256_set1_epi64x,
        _mm256_sll_epi64, _mm256_srl_epi64,
    };

    const LANES: usize = 4;

    /// AVX2 backend: same as SSE2 but with 4-lane (256-bit) vectors.
    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn find_first<F>(
        haystack: &[u64],
        needle_first: u64,
        needle_mask: u64,
        last_start: usize,
        word_limit: usize,
        verify: &mut F,
    ) -> Option<usize>
    where
        F: FnMut(usize) -> bool,
    {
        let needle = unsafe { _mm256_set1_epi64x(needle_first as i64) };
        let mask = unsafe { _mm256_set1_epi64x(needle_mask as i64) };

        for shift in 0..WORD_BITS {
            let mut i = 0;
            while i + LANES <= word_limit {
                if i * WORD_BITS + shift > last_start {
                    break;
                }

                let window = if shift == 0 {
                    unsafe { _mm256_loadu_si256(haystack.as_ptr().add(i).cast::<__m256i>()) }
                } else {
                    let src = haystack.as_ptr();
                    let w0 = unsafe { _mm256_loadu_si256(src.add(i).cast::<__m256i>()) };
                    let w1 = unsafe { _mm256_loadu_si256(src.add(i + 1).cast::<__m256i>()) };
                    let count_lo = unsafe { _mm_set1_epi64x(shift as i64) };
                    let count_hi = unsafe { _mm_set1_epi64x((WORD_BITS - shift) as i64) };
                    let lo = unsafe { _mm256_srl_epi64(w0, count_lo) };
                    let hi = unsafe { _mm256_sll_epi64(w1, count_hi) };
                    unsafe { _mm256_or_si256(lo, hi) }
                };

                let masked = unsafe { _mm256_and_si256(window, mask) };
                let cmp = unsafe { _mm256_cmpeq_epi64(masked, needle) };
                let hits = unsafe { _mm256_movemask_pd(core::mem::transmute(cmp)) } as u32;

                if hits != 0 {
                    for k in 0..LANES {
                        if hits & (1 << k) != 0 {
                            let pos = (i + k) * WORD_BITS + shift;
                            if pos <= last_start && verify(pos) {
                                return Some(pos);
                            }
                        }
                    }
                }

                i += LANES;
            }

            for j in i..word_limit {
                let pos = j * WORD_BITS + shift;
                if pos > last_start {
                    break;
                }
                let window = if shift == 0 {
                    haystack[j]
                } else {
                    let w0 = haystack[j];
                    let w1 = haystack.get(j + 1).copied().unwrap_or(0);
                    (w0 >> shift) | (w1 << (WORD_BITS - shift))
                };
                if (window & needle_mask) == needle_first && verify(pos) {
                    return Some(pos);
                }
            }
        }

        None
    }
}

// ---------------------------------------------------------------------------
// NEON — 2 haystack words per iteration
// ---------------------------------------------------------------------------

#[allow(unused)]
#[cfg(target_arch = "aarch64")]
mod neon {
    use super::*;

    use core::arch::aarch64::{
        uint64x2_t, vandq_u64, vceqq_u64, vdupq_n_u64, vgetq_lane_u64, vld1q_u64,
    };

    const LANES: usize = 2;

    /// NEON backend: 2-lane comparison for aarch64.  Unaligned windows
    /// fall back to scalar per-position computation.
    #[target_feature(enable = "neon")]
    pub(super) unsafe fn find_first<F>(
        haystack: &[u64],
        needle_first: u64,
        needle_mask: u64,
        last_start: usize,
        word_limit: usize,
        verify: &mut F,
    ) -> Option<usize>
    where
        F: FnMut(usize) -> bool,
    {
        let needle = unsafe { vdupq_n_u64(needle_first) };
        let mask = unsafe { vdupq_n_u64(needle_mask) };

        for shift in 0..WORD_BITS {
            let mut i = 0;
            while i < word_limit {
                if i * WORD_BITS + shift > last_start {
                    break;
                }

                if shift == 0 {
                    let window = unsafe { vld1q_u64(haystack.as_ptr().add(i)) };
                    let masked = unsafe { vandq_u64(window, mask) };
                    let cmp = unsafe { vceqq_u64(masked, needle) };
                    if unsafe { vgetq_lane_u64(cmp, 0) } != 0 {
                        let pos = i * WORD_BITS + shift;
                        if pos <= last_start && verify(pos) {
                            return Some(pos);
                        }
                    }
                    if unsafe { vgetq_lane_u64(cmp, 1) } != 0 {
                        let pos = (i + 1) * WORD_BITS + shift;
                        if pos <= last_start && verify(pos) {
                            return Some(pos);
                        }
                    }
                } else {
                    for k in 0..LANES {
                        let pos = (i + k) * WORD_BITS + shift;
                        if pos > last_start {
                            break;
                        }
                        let w0 = haystack[i + k];
                        let w1 = haystack.get(i + k + 1).copied().unwrap_or(0);
                        let window = (w0 >> shift) | (w1 << (WORD_BITS - shift));
                        if (window & needle_mask) == needle_first && verify(pos) {
                            return Some(pos);
                        }
                    }
                }

                i += LANES;
            }

            for j in i..word_limit {
                let pos = j * WORD_BITS + shift;
                if pos > last_start {
                    break;
                }
                let window = if shift == 0 {
                    haystack[j]
                } else {
                    let w0 = haystack[j];
                    let w1 = haystack.get(j + 1).copied().unwrap_or(0);
                    (w0 >> shift) | (w1 << (WORD_BITS - shift))
                };
                if (window & needle_mask) == needle_first && verify(pos) {
                    return Some(pos);
                }
            }
        }

        None
    }
}
