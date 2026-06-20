//! SIMD first-word pre-filter for `find`.
//!
//! For each bit offset (shift) in 0..64 we scan the haystack word-by-word,
//! comparing a sliding 64-bit window against the needle's first word.
//! Windows that match are then verified with a full `bits_equal_at` call.

use crate::WORD_BITS;
use crate::funcs_for_bits::SMALL_WORDS;

// ---------------------------------------------------------------------------
// Entry point — dispatches to the best available backend
// ---------------------------------------------------------------------------

/// Returns the position of the first verified match, or `None`.
#[inline]
pub(super) fn find_first_word<F>(
    haystack: &[u64],
    _bit_len: usize,
    needle_first: u64,
    needle_mask: u64,
    last_start: usize,
    verify: &mut F,
) -> Option<usize>
where
    F: FnMut(usize) -> bool,
{
    // For tiny haystacks, scalar beats SIMD setup overhead.
    if haystack.len() < SMALL_WORDS {
        return scalar_find(haystack, needle_first, needle_mask, last_start, verify);
    }

    // SIMD dispatch: each backend returns Some(pos) on a verified match.
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
    // Fallback: scalar
    scalar_find(haystack, needle_first, needle_mask, last_start, verify)
}

// ---------------------------------------------------------------------------
// Scalar fallback
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
    for shift in 0..WORD_BITS {
        for i in 0..haystack.len() {
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
// SSE2 backend (2 lanes)
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
        let chunk_end = haystack.len().saturating_sub(1);

        for shift in 0..WORD_BITS {
            let mut i = 0;
            while i < chunk_end {
                let pos0 = i * WORD_BITS + shift;
                if pos0 > last_start {
                    break;
                }

                let window = if shift == 0 {
                    unsafe { _mm_loadu_si128(haystack.as_ptr().add(i).cast::<__m128i>()) }
                } else {
                    let src = haystack.as_ptr();
                    let w01 = unsafe { _mm_loadu_si128(src.add(i).cast::<__m128i>()) };
                    let w12 = unsafe { _mm_loadu_si128(src.add(i + 1).cast::<__m128i>()) };
                    let count_lo = _mm_set1_epi64x(shift as i64);
                    let count_hi = _mm_set1_epi64x((WORD_BITS - shift) as i64);
                    let lo = _mm_srl_epi64(w01, count_lo);
                    let hi = _mm_sll_epi64(w12, count_hi);
                    _mm_or_si128(lo, hi)
                };

                let masked = _mm_and_si128(window, mask);
                let cmp = unsafe { _mm_cmpeq_epi64(masked, needle) };
                let hits = _mm_movemask_epi8(cmp) as u32;

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

            // Tail
            for j in i..haystack.len() {
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
// AVX2 backend (4 lanes)
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
        let chunk_end = haystack.len().saturating_sub(1);

        for shift in 0..WORD_BITS {
            let mut i = 0;
            while i + LANES <= chunk_end {
                let pos0 = i * WORD_BITS + shift;
                if pos0 > last_start {
                    break;
                }

                let window = if shift == 0 {
                    unsafe { _mm256_loadu_si256(haystack.as_ptr().add(i).cast::<__m256i>()) }
                } else {
                    let src = haystack.as_ptr();
                    let w0 = unsafe { _mm256_loadu_si256(src.add(i).cast::<__m256i>()) };
                    let w1 = unsafe { _mm256_loadu_si256(src.add(i + 1).cast::<__m256i>()) };
                    let count_lo = _mm_set1_epi64x(shift as i64);
                    let count_hi = _mm_set1_epi64x((WORD_BITS - shift) as i64);
                    let lo = _mm256_srl_epi64(w0, count_lo);
                    let hi = _mm256_sll_epi64(w1, count_hi);
                    _mm256_or_si256(lo, hi)
                };

                let masked = _mm256_and_si256(window, mask);
                let cmp = _mm256_cmpeq_epi64(masked, needle);
                let hits = _mm256_movemask_pd(unsafe { core::mem::transmute(cmp) }) as u32;

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

            // Tail
            for j in i..haystack.len() {
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
// NEON backend (2 lanes, aarch64)
// ---------------------------------------------------------------------------

#[cfg(target_arch = "aarch64")]
mod neon {
    use super::*;

    use core::arch::aarch64::{
        uint64x2_t, vandq_u64, vceqq_u64, vdupq_n_u64, vgetq_lane_u64, vld1q_u64,
    };

    const LANES: usize = 2;

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
        let chunk_end = haystack.len().saturating_sub(1);

        for shift in 0..WORD_BITS {
            let mut i = 0;
            while i < chunk_end {
                let pos0 = i * WORD_BITS + shift;
                if pos0 > last_start {
                    break;
                }

                if shift == 0 {
                    // Word-aligned: SIMD compare 2 lanes.
                    let window = vld1q_u64(haystack.as_ptr().add(i));
                    let masked = vandq_u64(window, mask);
                    let cmp = vceqq_u64(masked, needle);

                    if vgetq_lane_u64(cmp, 0) != 0 {
                        let pos = i * WORD_BITS + shift;
                        if pos <= last_start && verify(pos) {
                            return Some(pos);
                        }
                    }
                    if vgetq_lane_u64(cmp, 1) != 0 {
                        let pos = (i + 1) * WORD_BITS + shift;
                        if pos <= last_start && verify(pos) {
                            return Some(pos);
                        }
                    }
                } else {
                    // Unaligned: scalar path per position.
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

            // Tail
            for j in i..haystack.len() {
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
