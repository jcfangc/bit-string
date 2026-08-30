use super::*;

#[cfg(target_arch = "x86")]
use core::arch::x86::{
    __m128i, _mm_and_si128, _mm_loadu_si128, _mm_or_si128, _mm_storeu_si128, _mm_xor_si128,
};

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{
    __m128i, _mm_and_si128, _mm_loadu_si128, _mm_or_si128, _mm_storeu_si128, _mm_xor_si128,
};

const LANES: usize = 2;

#[inline]
fn apply<const OP: u8>(lhs: __m128i, rhs: __m128i) -> __m128i {
    match OP {
        OP_AND => unsafe { _mm_and_si128(lhs, rhs) },
        OP_OR => unsafe { _mm_or_si128(lhs, rhs) },
        OP_XOR => unsafe { _mm_xor_si128(lhs, rhs) },
        _ => unreachable!("unsupported binary bit operation"),
    }
}

/// SSE2 backend for `dst[i] = lhs[i] OP rhs[i]`.
///
/// Supports `dst == lhs`.
///
/// # Safety
///
/// - Caller must only call this when SSE2 is available.
/// - `dst` must be valid for writes of `len` initialized `u64` values.
/// - `lhs` and `rhs` must be valid for reads of `len` initialized `u64` values.
/// - `dst` must either not overlap `lhs`, or be exactly equal to `lhs`.
/// - `dst` must not overlap `rhs`.
#[target_feature(enable = "sse2")]
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
        // - Unaligned load/store intrinsics permit unaligned access.
        // - Pointer validity and overlap constraints are guaranteed by the caller.
        // - `dst == lhs` is safe because loads happen before the store.
        unsafe {
            let lhs_vec = _mm_loadu_si128(lhs.add(offset).cast::<__m128i>());
            let rhs_vec = _mm_loadu_si128(rhs.add(offset).cast::<__m128i>());
            let out_vec = apply::<OP>(lhs_vec, rhs_vec);

            _mm_storeu_si128(dst.add(offset).cast::<__m128i>(), out_vec);
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
