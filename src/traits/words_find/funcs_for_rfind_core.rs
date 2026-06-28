//! SIMD first-word pre-filter for `rfind`.
//!
//! Scanning order is **word-outer reverse, shift-inner reverse** so
//! positions are visited in decreasing order and `rfind` returns the
//! rightmost match.

use crate::SMALL_WORDS;
use crate::WORD_BITS;
use crate::funcs_for_bits::low_mask;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[inline]
pub(super) fn find_last_word<F>(
    haystack: &[u64],
    haystack_bit_len: usize,
    needle_words: &[u64],
    needle_bit_len: usize,
    verify: &mut F,
) -> Option<usize>
where
    F: FnMut(usize) -> bool,
{
    let needle_key = needle_words[0];
    let needle_mask = low_mask(needle_bit_len.min(WORD_BITS));
    let last_start = haystack_bit_len - needle_bit_len;

    if haystack.len() < SMALL_WORDS {
        return scalar_rfind(haystack, needle_key, needle_mask, last_start, verify);
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    {
        unsafe {
            return avx2::rfind(haystack, needle_key, needle_mask, last_start, verify);
        }
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse4.1",
        not(target_feature = "avx2")
    ))]
    {
        unsafe {
            return sse41::rfind(haystack, needle_key, needle_mask, last_start, verify);
        }
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        unsafe {
            return neon::rfind(haystack, needle_key, needle_mask, last_start, verify);
        }
    }

    #[allow(unused)]
    scalar_rfind(haystack, needle_key, needle_mask, last_start, verify)
}

// ---------------------------------------------------------------------------
// Scalar
// ---------------------------------------------------------------------------

fn scalar_rfind<F>(
    haystack: &[u64],
    needle_key: u64,
    needle_mask: u64,
    last_start: usize,
    verify: &mut F,
) -> Option<usize>
where
    F: FnMut(usize) -> bool,
{
    let start_word = (last_start / WORD_BITS).min(haystack.len().saturating_sub(1));
    for i in (0..=start_word).rev() {
        let base = i * WORD_BITS;
        let w0 = haystack[i];
        let w1 = haystack.get(i + 1).copied().unwrap_or(0);
        // Note: the SIMD backends compute max_shift differently —
        // `WORD_BITS.min(last_start - base + 1)` — to process
        // shifts in SIMD-sized chunks (2 or 4), relying on
        // `pos <= last_start` to skip out-of-range positions.
        let max_shift = (last_start - base).min(WORD_BITS - 1);
        for shift in (0..=max_shift).rev() {
            let pos = base + shift;
            let window = if shift == 0 {
                w0
            } else {
                (w0 >> shift) | (w1 << (WORD_BITS - shift))
            };
            if (window & needle_mask) == needle_key && verify(pos) {
                return Some(pos);
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// SSE2 — 2 consecutive shifts at once, reverse
// ---------------------------------------------------------------------------

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse41 {
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

    #[target_feature(enable = "sse4.1")]
    pub(super) unsafe fn rfind<F>(
        haystack: &[u64],
        needle_key: u64,
        needle_mask: u64,
        last_start: usize,
        verify: &mut F,
    ) -> Option<usize>
    where
        F: FnMut(usize) -> bool,
    {
        let needle = _mm_set1_epi64x(needle_key as i64);
        let mask = _mm_set1_epi64x(needle_mask as i64);

        let start_word = (last_start / WORD_BITS).min(haystack.len().saturating_sub(1));
        for i in (0..=start_word).rev() {
            let base = i * WORD_BITS;
            let w0 = haystack[i];
            let w1 = haystack.get(i + 1).copied().unwrap_or(0);
            let max_shift = WORD_BITS.min(last_start - base + 1);

            // Round up to a multiple of 2 so the SIMD loop
            // processes shifts in 2-lane pairs.  Out-of-range
            // positions are guarded by `pos <= last_start`.
            let mut s = max_shift.next_multiple_of(2).min(WORD_BITS);
            while s > 0 {
                s -= 2;

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
                let windows = unsafe { _mm_loadu_si128([win0, win1].as_ptr().cast::<__m128i>()) };
                let m = _mm_and_si128(windows, mask);
                let c = unsafe { _mm_cmpeq_epi64(m, needle) };
                let hits = _mm_movemask_epi8(c) as u32;

                // Check higher shift (lane 1) first for rightmost.
                if hits & 0xff00 != 0 {
                    let pos = base + s + 1;
                    if pos <= last_start && verify(pos) {
                        return Some(pos);
                    }
                }
                if hits & 0xff != 0 {
                    let pos = base + s;
                    if pos <= last_start && verify(pos) {
                        return Some(pos);
                    }
                }
            }
        }

        None
    }
}

// ---------------------------------------------------------------------------
// AVX2 — 4 consecutive shifts at once, reverse
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
    pub(super) unsafe fn rfind<F>(
        haystack: &[u64],
        needle_key: u64,
        needle_mask: u64,
        last_start: usize,
        verify: &mut F,
    ) -> Option<usize>
    where
        F: FnMut(usize) -> bool,
    {
        let needle = _mm256_set1_epi64x(needle_key as i64);
        let mask = _mm256_set1_epi64x(needle_mask as i64);

        let start_word = (last_start / WORD_BITS).min(haystack.len().saturating_sub(1));
        for i in (0..=start_word).rev() {
            let base = i * WORD_BITS;
            let w0 = haystack[i];
            let w1 = haystack.get(i + 1).copied().unwrap_or(0);
            let max_shift = WORD_BITS.min(last_start - base + 1);

            // Round up to a multiple of 4 so the SIMD loop
            // processes shifts in 4-lane groups.  Out-of-range
            // positions are guarded by `pos <= last_start`.
            let mut s = max_shift.next_multiple_of(4).min(WORD_BITS);
            while s > 0 {
                s -= 4;

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
                let c = unsafe { _mm256_cmpeq_epi64(m, needle) };
                let hits = unsafe { _mm256_movemask_pd(core::mem::transmute(c)) } as u32;

                // Check higher shifts first for rightmost.
                if hits != 0 {
                    for k in (0..(end - s)).rev() {
                        if hits & (1 << k) != 0 {
                            let pos = base + s + k;
                            if pos <= last_start && verify(pos) {
                                return Some(pos);
                            }
                        }
                    }
                }
            }
        }

        None
    }
}

// ---------------------------------------------------------------------------
// NEON — 2 consecutive shifts at once, reverse
// ---------------------------------------------------------------------------

#[allow(unused)]
#[cfg(target_arch = "aarch64")]
mod neon {
    use super::*;

    use core::arch::aarch64::{
        uint64x2_t, vandq_u64, vceqq_u64, vdupq_n_u64, vgetq_lane_u64, vld1q_u64,
    };

    #[target_feature(enable = "neon")]
    pub(super) unsafe fn rfind<F>(
        haystack: &[u64],
        needle_key: u64,
        needle_mask: u64,
        last_start: usize,
        verify: &mut F,
    ) -> Option<usize>
    where
        F: FnMut(usize) -> bool,
    {
        let needle = vdupq_n_u64(needle_key);
        let mask = vdupq_n_u64(needle_mask);

        let start_word = (last_start / WORD_BITS).min(haystack.len().saturating_sub(1));
        for i in (0..=start_word).rev() {
            let base = i * WORD_BITS;
            let w0 = haystack[i];
            let w1 = haystack.get(i + 1).copied().unwrap_or(0);
            let max_shift = WORD_BITS.min(last_start - base + 1);

            // Round up to a multiple of 2 so the SIMD loop
            // processes shifts in 2-lane pairs.  Out-of-range
            // positions are guarded by `pos <= last_start`.
            let mut s = max_shift.next_multiple_of(2).min(WORD_BITS);
            while s > 0 {
                s -= 2;

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
                let windows = unsafe { vld1q_u64(wins.as_ptr()) };
                let m = vandq_u64(windows, mask);
                let c = vceqq_u64(m, needle);

                // Check higher shift (lane 1) first for rightmost.
                if vgetq_lane_u64(c, 1) != 0 {
                    let pos = base + s + 1;
                    if pos <= last_start && verify(pos) {
                        return Some(pos);
                    }
                }
                if vgetq_lane_u64(c, 0) != 0 {
                    let pos = base + s;
                    if pos <= last_start && verify(pos) {
                        return Some(pos);
                    }
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests_for_backend_equivalence;
