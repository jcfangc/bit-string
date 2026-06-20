//! SIMD first-word pre-filter for `find`.
//!
//! Scanning order is **word-outer, shift-inner** so positions are visited
//! in increasing order and `find` returns the earliest match.

use crate::SMALL_WORDS;
use crate::WORD_BITS;
use crate::funcs_for_bits::low_mask;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[inline]
pub(super) fn find_first_word<F>(
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
    if haystack.len() < SMALL_WORDS {
        return scalar_find(haystack, needle_first, needle_mask, last_start, verify);
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    {
        unsafe {
            return avx2::find(haystack, needle_first, needle_mask, last_start, verify);
        }
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse2",
        not(target_feature = "avx2")
    ))]
    {
        unsafe {
            return sse2::find(haystack, needle_first, needle_mask, last_start, verify);
        }
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        unsafe {
            return neon::find(haystack, needle_first, needle_mask, last_start, verify);
        }
    }

    #[allow(unused)]
    scalar_find(haystack, needle_first, needle_mask, last_start, verify)
}

// ---------------------------------------------------------------------------
// Scalar
// ---------------------------------------------------------------------------

fn scalar_find<F>(
    haystack: &[u64],
    needle_first: u64,
    needle_mask: u64,
    last_start: usize,
    verify: &mut F,
) -> Option<usize>
where
    F: FnMut(usize) -> bool,
{
    for i in 0..haystack.len() {
        let w0 = haystack[i];
        let w1 = haystack.get(i + 1).copied().unwrap_or(0);
        for shift in 0..WORD_BITS {
            let pos = i * WORD_BITS + shift;
            if pos > last_start {
                break;
            }
            let window = if shift == 0 {
                w0
            } else {
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
// SSE2 — same loop as scalar but checks 2 consecutive shifts at once
// ---------------------------------------------------------------------------

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse2 {
    use super::*;

    #[cfg(target_arch = "x86")]
    use core::arch::x86::{
        __m128i, _mm_and_si128, _mm_cmpeq_epi64, _mm_loadu_si128, _mm_movemask_epi8,
        _mm_set1_epi64x,
    };
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m128i, _mm_and_si128, _mm_cmpeq_epi64, _mm_loadu_si128, _mm_movemask_epi8,
        _mm_set1_epi64x,
    };

    #[target_feature(enable = "sse2")]
    pub(super) unsafe fn find<F>(
        haystack: &[u64],
        needle_first: u64,
        needle_mask: u64,
        last_start: usize,
        verify: &mut F,
    ) -> Option<usize>
    where
        F: FnMut(usize) -> bool,
    {
        let needle = _mm_set1_epi64x(needle_first as i64);
        let mask = _mm_set1_epi64x(needle_mask as i64);

        for i in 0..haystack.len() {
            let base = i * WORD_BITS;
            if base > last_start {
                break;
            }
            let w0 = haystack[i];
            let w1 = haystack.get(i + 1).copied().unwrap_or(0);

            let mut s = 0;
            while s < WORD_BITS {
                if base + s > last_start {
                    break;
                }
                // Build two consecutive windows manually, pack into __m128i.
                let win0 = if s == 0 {
                    w0
                } else {
                    (w0 >> s) | (w1 << (WORD_BITS - s))
                };
                let win1 = if s + 1 >= WORD_BITS {
                    0
                } else {
                    (w0 >> (s + 1)) | (w1 << (WORD_BITS - (s + 1)))
                };
                let windows = _mm_set1_epi64x(win0 as i64);
                let windows = unsafe { _mm_loadu_si128([win0, win1].as_ptr().cast::<__m128i>()) };
                let m = _mm_and_si128(windows, mask);
                let c = unsafe { _mm_cmpeq_epi64(m, needle) };
                let hits = _mm_movemask_epi8(c) as u32;

                if hits & 0xff != 0 {
                    let pos = base + s;
                    if pos <= last_start && verify(pos) {
                        return Some(pos);
                    }
                }
                if hits & 0xff00 != 0 {
                    let pos = base + s + 1;
                    if pos <= last_start && verify(pos) {
                        return Some(pos);
                    }
                }

                s += 2;
            }
        }

        None
    }
}

// ---------------------------------------------------------------------------
// AVX2 — checks 4 consecutive shifts at once
// ---------------------------------------------------------------------------

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2 {
    use super::*;

    #[cfg(target_arch = "x86")]
    use core::arch::x86::{
        __m256i, _mm256_and_si256, _mm256_cmpeq_epi64, _mm256_loadu_si256, _mm256_movemask_pd,
        _mm256_set1_epi64x,
    };
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m256i, _mm256_and_si256, _mm256_cmpeq_epi64, _mm256_loadu_si256, _mm256_movemask_pd,
        _mm256_set1_epi64x,
    };

    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn find<F>(
        haystack: &[u64],
        needle_first: u64,
        needle_mask: u64,
        last_start: usize,
        verify: &mut F,
    ) -> Option<usize>
    where
        F: FnMut(usize) -> bool,
    {
        let needle = _mm256_set1_epi64x(needle_first as i64);
        let mask = _mm256_set1_epi64x(needle_mask as i64);

        for i in 0..haystack.len() {
            let base = i * WORD_BITS;
            if base > last_start {
                break;
            }
            let w0 = haystack[i];
            let w1 = haystack.get(i + 1).copied().unwrap_or(0);

            let mut s = 0;
            while s < WORD_BITS {
                if base + s > last_start {
                    break;
                }
                let end = WORD_BITS.min(s + 4);
                let mut wins = [0u64; 4];
                for k in 0..(end - s) {
                    let shift = s + k;
                    wins[k] = if shift == 0 {
                        w0
                    } else {
                        (w0 >> shift) | (w1 << (WORD_BITS - shift))
                    };
                }
                let windows = unsafe { _mm256_loadu_si256(wins.as_ptr().cast::<__m256i>()) };
                let m = _mm256_and_si256(windows, mask);
                let c = _mm256_cmpeq_epi64(m, needle);
                let hits = unsafe { _mm256_movemask_pd(core::mem::transmute(c)) } as u32;
                if hits != 0 {
                    for k in 0..(end - s) {
                        if hits & (1 << k) != 0 {
                            let pos = base + s + k;
                            if pos <= last_start && verify(pos) {
                                return Some(pos);
                            }
                        }
                    }
                }

                s += 4;
            }
        }

        None
    }
}

// ---------------------------------------------------------------------------
// NEON
// ---------------------------------------------------------------------------

#[allow(unused)]
#[cfg(target_arch = "aarch64")]
mod neon {
    use super::*;

    use core::arch::aarch64::{
        uint64x2_t, vandq_u64, vceqq_u64, vdupq_n_u64, vgetq_lane_u64, vld1q_u64,
    };

    #[target_feature(enable = "neon")]
    pub(super) unsafe fn find<F>(
        haystack: &[u64],
        needle_first: u64,
        needle_mask: u64,
        last_start: usize,
        verify: &mut F,
    ) -> Option<usize>
    where
        F: FnMut(usize) -> bool,
    {
        let needle = vdupq_n_u64(needle_first);
        let mask = vdupq_n_u64(needle_mask);

        for i in 0..haystack.len() {
            let base = i * WORD_BITS;
            if base > last_start {
                break;
            }
            let w0 = haystack[i];
            let w1 = haystack.get(i + 1).copied().unwrap_or(0);

            let mut s = 0;
            while s < WORD_BITS {
                if base + s > last_start {
                    break;
                }
                let end = WORD_BITS.min(s + 2);
                let mut wins = [0u64; 2];
                for k in 0..(end - s) {
                    let shift = s + k;
                    wins[k] = if shift == 0 {
                        w0
                    } else {
                        (w0 >> shift) | (w1 << (WORD_BITS - shift))
                    };
                }
                let windows = vld1q_u64(wins.as_ptr());
                let m = vandq_u64(windows, mask);
                let c = vceqq_u64(m, needle);
                if vgetq_lane_u64(c, 0) != 0 {
                    let pos = base + s;
                    if pos <= last_start && verify(pos) {
                        return Some(pos);
                    }
                }
                if vgetq_lane_u64(c, 1) != 0 && s + 1 < end {
                    let pos = base + s + 1;
                    if pos <= last_start && verify(pos) {
                        return Some(pos);
                    }
                }

                s += 2;
            }
        }

        None
    }
}

#[cfg(test)]
mod tests_for_backend_equivalence;
