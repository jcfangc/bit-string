// ═══════════════════════════════════════════════════════════════════════
// NEON backend — 128-bit / 2-lane, raw intrinsics (no chunk_eq dispatch).
// ═══════════════════════════════════════════════════════════════════════

use core::arch::aarch64::{uint64x2_t, vceqq_u64, vdupq_n_u64, vgetq_lane_u64, vld1q_u64};

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

/// NEON forward scan: advances `p` past all-FILL chunks.
///
/// # Safety
///
/// Caller must ensure NEON is available.
/// `p` through `end` must be valid for u64 reads.
#[target_feature(enable = "neon")]
pub(super) unsafe fn leading_scan<const FILL: u64>(
    mut p: *const u64,
    end: *const u64,
    total: usize,
) -> *const u64 {
    // SAFETY: only callable when NEON is available.  All pointer
    // arithmetic stays within `[p, end)`.
    unsafe {
        let mut iters = total / LANES_2X;
        while iters > 0 {
            if !chunk_eq::<FILL>(p) || !chunk_eq::<FILL>(p.add(LANES)) {
                return p;
            }
            p = p.add(LANES_2X);
            iters -= 1;
        }
        let limit = end.sub(LANES);
        while p <= limit {
            if !chunk_eq::<FILL>(p) {
                break;
            }
            p = p.add(LANES);
        }
        p
    }
}
