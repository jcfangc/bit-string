use super::*;

use core::arch::aarch64::{
    uint64x2_t, vandq_u64, vceqq_u64, vdupq_n_u64, vgetq_lane_u64, vld1q_u64,
};

const LANES: usize = 2;

/// NEON backend: 2-lane comparison for aarch64.  Unaligned windows
/// fall back to scalar per-position computation.
#[target_feature(enable = "neon")]
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
