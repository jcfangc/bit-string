// ═══════════════════════════════════════════════════════════════════════
// NEON backend — 128-bit / 2-lane, raw intrinsics (no chunk_eq dispatch).
// ═══════════════════════════════════════════════════════════════════════

use core::arch::aarch64::{vceqq_u64, vdupq_n_u64, vgetq_lane_u64, vld1q_u64};

const LANES: usize = 2;
const LANES_2X: usize = LANES * 2;

#[inline(always)]
unsafe fn chunk_eq<const FILL: u64>(ptr: *const u64) -> bool {
    // SAFETY: caller ensures `ptr` is valid for 2 u64 reads and
    // NEON is available.
    unsafe {
        let data = vld1q_u64(ptr);
        let cmp = vceqq_u64(data, vdupq_n_u64(FILL));
        vgetq_lane_u64(cmp, 0) != 0 && vgetq_lane_u64(cmp, 1) != 0
    }
}

/// NEON reverse scan: advances `done` past all-FILL chunks from the right.
///
/// # Safety
///
/// Caller must ensure NEON is available.
/// `ptr` through `ptr.add(wi_end + 1)` must be valid for u64 reads.
#[target_feature(enable = "neon")]
pub(super) unsafe fn trailing_scan<const FILL: u64>(
    ptr: *const u64,
    wi_end: usize,
    mut done: usize,
    total_words: usize,
) -> usize {
    // SAFETY: only callable when NEON is available.  All pointer
    // arithmetic stays within bounds.
    unsafe {
        while done + LANES_2X <= total_words {
            let chunk_start = wi_end + 1 - (done + LANES_2X);
            if !chunk_eq::<FILL>(ptr.add(chunk_start))
                || !chunk_eq::<FILL>(ptr.add(chunk_start + LANES))
            {
                return done;
            }
            done += LANES_2X;
        }
        while done + LANES <= total_words {
            let chunk_start = wi_end + 1 - (done + LANES);
            if !chunk_eq::<FILL>(ptr.add(chunk_start)) {
                break;
            }
            done += LANES;
        }
        done
    }
}
