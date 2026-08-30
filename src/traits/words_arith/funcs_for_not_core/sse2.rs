use super::scalar;

#[cfg(target_arch = "x86")]
use core::arch::x86::{
    __m128i, _mm_cmpeq_epi8, _mm_loadu_si128, _mm_setzero_si128, _mm_storeu_si128, _mm_xor_si128,
};

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{
    __m128i, _mm_cmpeq_epi8, _mm_loadu_si128, _mm_setzero_si128, _mm_storeu_si128, _mm_xor_si128,
};

const LANES: usize = 2;

/// SSE2 backend for `dst[i] = !src[i]`.
///
/// Supports `dst == src`.
///
/// # Safety
///
/// - Caller must only call this when SSE2 is available.
/// - `dst` must be valid for writes of `len` initialized `u64` values.
/// - `src` must be valid for reads of `len` initialized `u64` values.
/// - `dst` must either not overlap `src`, or be exactly equal to `src`.
#[target_feature(enable = "sse2")]
pub(super) unsafe fn words(dst: *mut u64, src: *const u64, len: usize) {
    let chunks = len / LANES;

    for chunk in 0..chunks {
        let offset = chunk * LANES;

        // SAFETY:
        // - `offset + LANES <= len`.
        // - `_mm_loadu_si128` and `_mm_storeu_si128` permit unaligned access.
        // - Pointer validity and overlap constraints are guaranteed by the caller.
        // - `dst == src` is safe because load happens before store.
        unsafe {
            let zero = _mm_setzero_si128();
            let all_ones = _mm_cmpeq_epi8(zero, zero);
            let src_vec = _mm_loadu_si128(src.add(offset).cast::<__m128i>());
            let out_vec = _mm_xor_si128(src_vec, all_ones);

            _mm_storeu_si128(dst.add(offset).cast::<__m128i>(), out_vec);
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
