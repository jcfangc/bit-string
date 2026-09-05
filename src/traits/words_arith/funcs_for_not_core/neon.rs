use super::scalar;

use core::arch::aarch64::{uint64x2_t, vdupq_n_u64, veorq_u64, vld1q_u64, vst1q_u64};

const LANES: usize = 2;

#[inline]
fn not_vec(src: uint64x2_t) -> uint64x2_t {
    // SAFETY:
    // - This helper is only called from `words`, which has
    //   `#[target_feature(enable = "neon")]`.
    // - The dispatch path only reaches `words` when NEON is enabled.
    unsafe { veorq_u64(src, vdupq_n_u64(u64::MAX)) }
}

/// NEON backend for `dst[i] = !src[i]`.
///
/// Supports `dst == src`.
///
/// # Safety
///
/// - Caller must only call this when NEON is available.
/// - `dst` must be valid for writes of `len` initialized `u64` values.
/// - `src` must be valid for reads of `len` initialized `u64` values.
/// - `dst` must either not overlap `src`, or be exactly equal to `src`.
#[target_feature(enable = "neon")]
pub(super) unsafe fn words(dst: *mut u64, src: *const u64, len: usize) {
    let chunks = len / LANES;

    for chunk in 0..chunks {
        let offset = chunk * LANES;

        // SAFETY:
        // - `offset + LANES <= len`.
        // - Each NEON vector reads/writes 2 u64 values.
        // - Pointer validity and overlap constraints are guaranteed by the caller.
        // - `dst == src` is safe because load happens before store.
        unsafe {
            let src_vec = vld1q_u64(src.add(offset));
            let out_vec = not_vec(src_vec);

            vst1q_u64(dst.add(offset), out_vec);
        }
    }

    let done = chunks * LANES;

    // SAFETY:
    // - `done <= len`.
    // - Tail range is `done..len`.
    // - Pointer validity and overlap constraints are guaranteed by the caller.
    unsafe {
        scalar::words(dst.add(done), src.add(done), len - done);
    }
}
