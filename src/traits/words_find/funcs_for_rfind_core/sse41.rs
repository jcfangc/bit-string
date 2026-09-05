// ---------------------------------------------------------------------------
// SSE4.1 — 2 consecutive shifts at once, reverse
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
