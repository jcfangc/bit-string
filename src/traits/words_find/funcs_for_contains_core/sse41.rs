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

/// SSE4.1 backend: loads 2 consecutive words, computes a sliding
/// window for the current shift, and compares against the broadcast
/// needle word.  `movemask` extracts match lanes.
#[target_feature(enable = "sse4.1")]
pub(super) unsafe fn find_any<F>(
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
