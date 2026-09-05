use super::*;

use core::arch::aarch64::{uint64x2_t, vandq_u64, veorq_u64, vld1q_u64, vorrq_u64, vst1q_u64};

const LANES: usize = 2;

#[inline]
fn apply<const OP: u8>(lhs: uint64x2_t, rhs: uint64x2_t) -> uint64x2_t {
    // SAFETY:
    // - This helper is only called from `words`, which has
    //   `#[target_feature(enable = "neon")]`.
    // - The dispatch path only reaches `words` when NEON is enabled.
    unsafe {
        match OP {
            OP_AND => vandq_u64(lhs, rhs),
            OP_OR => vorrq_u64(lhs, rhs),
            OP_XOR => veorq_u64(lhs, rhs),
            _ => unreachable!("unsupported binary bit operation"),
        }
    }
}

/// NEON backend for `dst[i] = lhs[i] OP rhs[i]`.
///
/// Supports `dst == lhs`.
///
/// # Safety
///
/// - Caller must only call this when NEON is available.
/// - `dst` must be valid for writes of `len` initialized `u64` values.
/// - `lhs` and `rhs` must be valid for reads of `len` initialized `u64` values.
/// - `dst` must either not overlap `lhs`, or be exactly equal to `lhs`.
/// - `dst` must not overlap `rhs`.
#[target_feature(enable = "neon")]
pub(super) unsafe fn words<const OP: u8>(
    dst: *mut u64,
    lhs: *const u64,
    rhs: *const u64,
    len: usize,
) {
    let chunks = len / LANES;

    for chunk in 0..chunks {
        let offset = chunk * LANES;

        // SAFETY:
        // - `offset + LANES <= len`.
        // - Each NEON vector reads/writes 2 u64 values.
        // - Pointer validity and overlap constraints are guaranteed by the caller.
        // - `dst == lhs` is safe because both loads happen before the store.
        unsafe {
            let lhs_vec = vld1q_u64(lhs.add(offset));
            let rhs_vec = vld1q_u64(rhs.add(offset));
            let out_vec = apply::<OP>(lhs_vec, rhs_vec);

            vst1q_u64(dst.add(offset), out_vec);
        }
    }

    let done = chunks * LANES;

    // SAFETY:
    // - `done <= len`.
    // - Tail range is `done..len`.
    // - Pointer validity and overlap constraints are guaranteed by the caller.
    unsafe {
        scalar::words::<OP>(dst.add(done), lhs.add(done), rhs.add(done), len - done);
    }
}
