// ---------------------------------------------------------------------------
// NEON — 2 consecutive shifts at once, reverse
// ---------------------------------------------------------------------------

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
