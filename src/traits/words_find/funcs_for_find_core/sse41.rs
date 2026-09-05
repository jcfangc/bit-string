// ---------------------------------------------------------------------------
// SSE4.1 — same loop as scalar but checks 2 consecutive shifts at once
// ---------------------------------------------------------------------------

use super::*;

#[cfg(target_arch = "x86")]
use core::arch::x86::{
    __m128i, _mm_and_si128, _mm_cmpeq_epi64, _mm_loadu_si128, _mm_movemask_epi8, _mm_set1_epi64x,
};
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{
    __m128i, _mm_and_si128, _mm_cmpeq_epi64, _mm_loadu_si128, _mm_movemask_epi8, _mm_set1_epi64x,
};

#[target_feature(enable = "sse4.1")]
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
