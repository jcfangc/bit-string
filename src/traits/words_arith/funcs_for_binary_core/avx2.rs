use super::*;

#[cfg(target_arch = "x86")]
use core::arch::x86::{
    __m256i, _mm256_and_si256, _mm256_loadu_si256, _mm256_or_si256, _mm256_storeu_si256,
    _mm256_xor_si256,
};

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{
    __m256i, _mm256_and_si256, _mm256_loadu_si256, _mm256_or_si256, _mm256_storeu_si256,
    _mm256_xor_si256,
};

const LANES: usize = 4;

#[inline]
fn apply<const OP: u8>(lhs: __m256i, rhs: __m256i) -> __m256i {
    match OP {
        OP_AND => unsafe { _mm256_and_si256(lhs, rhs) },
        OP_OR => unsafe { _mm256_or_si256(lhs, rhs) },
        OP_XOR => unsafe { _mm256_xor_si256(lhs, rhs) },
        _ => unreachable!("unsupported binary bit operation"),
    }
}

/// AVX2 backend for `dst[i] = lhs[i] OP rhs[i]`.
///
/// Supports `dst == lhs`.
///
/// # Safety
///
/// - Caller must only call this when AVX2 is available.
/// - `dst` must be valid for writes of `len` initialized `u64` values.
/// - `lhs` and `rhs` must be valid for reads of `len` initialized `u64` values.
/// - `dst` must either not overlap `lhs`, or be exactly equal to `lhs`.
/// - `dst` must not overlap `rhs`.
#[target_feature(enable = "avx2")]
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
        // - `_mm256_loadu_si256` and `_mm256_storeu_si256` permit unaligned access.
        // - Pointer validity and overlap constraints are guaranteed by the caller.
        // - `dst == lhs` is safe because both loads happen before the store.
        unsafe {
            let lhs_vec = _mm256_loadu_si256(lhs.add(offset).cast::<__m256i>());
            let rhs_vec = _mm256_loadu_si256(rhs.add(offset).cast::<__m256i>());
            let out_vec = apply::<OP>(lhs_vec, rhs_vec);

            _mm256_storeu_si256(dst.add(offset).cast::<__m256i>(), out_vec);
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
