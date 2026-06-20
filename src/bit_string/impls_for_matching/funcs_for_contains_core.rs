//! SIMD first-word pre-filter for `contains`.
//!
//! Uses **shift-outer, word-inner** ordering because `contains` only
//! needs a yes/no answer — order doesn't matter.  This lets us process
//! LANES haystack words in parallel within each shift, maximizing SIMD
//! throughput.

use crate::SMALL_WORDS;
use crate::WORD_BITS;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Returns `true` if any 64-bit window in `haystack` matches
/// `needle_first` (after masking) AND `verify(pos)` succeeds.
///
/// Only positions `pos ∈ [0, last_start]` are considered.  Words beyond
/// `last_start / WORD_BITS + 1` are ignored.
///
/// Uses **shift-outer, word-inner** ordering: for each bit offset
/// (shift), all qualifying word pairs are scanned.  Within a shift,
/// the SIMD backends process `LANES` words in parallel.  This ordering
/// does **not** guarantee that the first match found is the earliest
/// position — use [`find_first_word`](super::funcs_for_find_core::find_first_word)
/// when position order matters.
#[inline]
pub(super) fn contains_first_word<F>(
    haystack: &[u64],
    needle_first: u64,
    needle_mask: u64,
    last_start: usize,
    verify: &mut F,
) -> bool
where
    F: FnMut(usize) -> bool,
{
    if haystack.len() < SMALL_WORDS {
        return scalar(haystack, needle_first, needle_mask, last_start, verify);
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    {
        unsafe {
            return avx2::contains(haystack, needle_first, needle_mask, last_start, verify);
        }
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse2",
        not(target_feature = "avx2")
    ))]
    {
        unsafe {
            return sse2::contains(haystack, needle_first, needle_mask, last_start, verify);
        }
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        unsafe {
            return neon::contains(haystack, needle_first, needle_mask, last_start, verify);
        }
    }

    #[allow(unused)]
    scalar(haystack, needle_first, needle_mask, last_start, verify)
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
    verify: &mut F,
) -> bool
where
    F: FnMut(usize) -> bool,
{
    // Only scan words that can contain a window starting at ≤ last_start.
    let max_word = last_start / WORD_BITS;
    let word_limit = (max_word + 1).min(haystack.len());

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
                return true;
            }
        }
    }
    false
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
    pub(super) unsafe fn contains<F>(
        haystack: &[u64],
        needle_first: u64,
        needle_mask: u64,
        last_start: usize,
        verify: &mut F,
    ) -> bool
    where
        F: FnMut(usize) -> bool,
    {
        let needle = unsafe { _mm_set1_epi64x(needle_first as i64) };
        let mask = unsafe { _mm_set1_epi64x(needle_mask as i64) };
        let max_word = last_start / WORD_BITS;
        let word_limit = (max_word + 1).min(haystack.len());

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
                        return true;
                    }
                }
                if hits & 0xff00 != 0 {
                    let pos = (i + 1) * WORD_BITS + shift;
                    if pos <= last_start && verify(pos) {
                        return true;
                    }
                }

                i += LANES;
            }

            // Tail
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
                    return true;
                }
            }
        }

        false
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
    pub(super) unsafe fn contains<F>(
        haystack: &[u64],
        needle_first: u64,
        needle_mask: u64,
        last_start: usize,
        verify: &mut F,
    ) -> bool
    where
        F: FnMut(usize) -> bool,
    {
        let needle = unsafe { _mm256_set1_epi64x(needle_first as i64) };
        let mask = unsafe { _mm256_set1_epi64x(needle_mask as i64) };
        let max_word = last_start / WORD_BITS;
        let word_limit = (max_word + 1).min(haystack.len());

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
                                return true;
                            }
                        }
                    }
                }

                i += LANES;
            }

            // Tail
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
                    return true;
                }
            }
        }

        false
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
    pub(super) unsafe fn contains<F>(
        haystack: &[u64],
        needle_first: u64,
        needle_mask: u64,
        last_start: usize,
        verify: &mut F,
    ) -> bool
    where
        F: FnMut(usize) -> bool,
    {
        let needle = unsafe { vdupq_n_u64(needle_first) };
        let mask = unsafe { vdupq_n_u64(needle_mask) };
        let max_word = last_start / WORD_BITS;
        let word_limit = (max_word + 1).min(haystack.len());

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
                            return true;
                        }
                    }
                    if unsafe { vgetq_lane_u64(cmp, 1) } != 0 {
                        let pos = (i + 1) * WORD_BITS + shift;
                        if pos <= last_start && verify(pos) {
                            return true;
                        }
                    }
                } else {
                    // Unaligned: scalar per position.
                    for k in 0..LANES {
                        let pos = (i + k) * WORD_BITS + shift;
                        if pos > last_start {
                            break;
                        }
                        let w0 = haystack[i + k];
                        let w1 = haystack.get(i + k + 1).copied().unwrap_or(0);
                        let window = (w0 >> shift) | (w1 << (WORD_BITS - shift));
                        if (window & needle_mask) == needle_first && verify(pos) {
                            return true;
                        }
                    }
                }

                i += LANES;
            }

            // Tail
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
                    return true;
                }
            }
        }

        false
    }
}
