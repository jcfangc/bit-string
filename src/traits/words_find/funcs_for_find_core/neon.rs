// ---------------------------------------------------------------------------
// NEON
// ---------------------------------------------------------------------------

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
            let windows = unsafe { vld1q_u64(wins.as_ptr()) };
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
