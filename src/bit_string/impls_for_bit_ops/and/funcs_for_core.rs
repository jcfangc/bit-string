use alloc::{boxed::Box, vec::Vec};

#[inline]
pub(super) fn owned(lhs: &[u64], rhs: &[u64]) -> Box<[u64]> {
    debug_assert_eq!(lhs.len(), rhs.len());

    let len = lhs.len();
    let mut out = Vec::<u64>::with_capacity(len);

    // SAFETY:
    // - `out` has capacity for exactly `len` u64 values.
    // - `lhs` and `rhs` are valid for reads of `len` u64 values.
    // - `out.as_mut_ptr()` is valid for writes of `len` u64 values.
    // - `out` is freshly allocated, so it does not overlap `lhs` or `rhs`.
    // - `dispatch` writes every slot in `0..len` exactly once.
    unsafe {
        dispatch(out.as_mut_ptr(), lhs.as_ptr(), rhs.as_ptr(), len);
        out.set_len(len);
    }

    out.into_boxed_slice()
}

#[inline]
pub(super) fn assign(lhs: &mut [u64], rhs: &[u64]) {
    debug_assert_eq!(lhs.len(), rhs.len());

    let len = lhs.len();
    let lhs_ptr = lhs.as_mut_ptr();

    // SAFETY:
    // - `lhs_ptr` is valid for reads and writes of `len` u64 values.
    // - `rhs` is valid for reads of `len` u64 values.
    // - `dst == lhs` is explicitly allowed by `dispatch`.
    // - Safe Rust prevents `rhs` from aliasing `lhs` in normal calls.
    // - `dispatch` writes every slot in `0..len` exactly once.
    unsafe {
        dispatch(lhs_ptr, lhs_ptr.cast_const(), rhs.as_ptr(), len);
    }
}

/// Writes `lhs[i] & rhs[i]` into `dst[i]` for every `i in 0..len`.
///
/// `dst` may be exactly equal to `lhs`, which enables in-place assignment.
/// Partial overlaps are not allowed.
///
/// # Safety
///
/// - `dst` must be valid for writes of `len` initialized `u64` values.
/// - `lhs` and `rhs` must be valid for reads of `len` initialized `u64` values.
/// - `dst` must either:
///   - not overlap `lhs`, or
///   - be exactly equal to `lhs`.
/// - `dst` must not overlap `rhs`.
#[inline]
unsafe fn dispatch(dst: *mut u64, lhs: *const u64, rhs: *const u64, len: usize) {
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    {
        // SAFETY:
        // - Forwarded from `dispatch`'s safety contract.
        // - This branch is compiled only when AVX2 is enabled.
        unsafe { avx2::and_words(dst, lhs, rhs, len) };
        return;
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse2"
    ))]
    {
        // SAFETY:
        // - Forwarded from `dispatch`'s safety contract.
        // - This branch is compiled only when SSE2 is enabled.
        unsafe { sse2::and_words(dst, lhs, rhs, len) };
        return;
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        // SAFETY:
        // - Forwarded from `dispatch`'s safety contract.
        // - This branch is compiled only when NEON is enabled.
        unsafe { neon::and_words(dst, lhs, rhs, len) };
        return;
    }

    #[allow(unused)]
    // SAFETY: Forwarded from `dispatch`'s safety contract.
    unsafe {
        scalar::and_words(dst, lhs, rhs, len)
    };
}

#[allow(unused)]
mod scalar {
    /// Scalar backend for `dst[i] = lhs[i] & rhs[i]`.
    ///
    /// Supports `dst == lhs`.
    ///
    /// # Safety
    ///
    /// - `dst` must be valid for writes of `len` initialized `u64` values.
    /// - `lhs` and `rhs` must be valid for reads of `len` initialized `u64` values.
    /// - `dst` must either not overlap `lhs`, or be exactly equal to `lhs`.
    /// - `dst` must not overlap `rhs`.
    #[inline]
    pub(super) unsafe fn and_words(dst: *mut u64, lhs: *const u64, rhs: *const u64, len: usize) {
        for i in 0..len {
            // SAFETY:
            // - `i < len`.
            // - Pointer validity and non-overlap are guaranteed by the caller.
            unsafe {
                dst.add(i).write(lhs.add(i).read() & rhs.add(i).read());
            }
        }
    }
}

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2 {
    use super::scalar;

    #[cfg(target_arch = "x86")]
    use core::arch::x86::{__m256i, _mm256_and_si256, _mm256_loadu_si256, _mm256_storeu_si256};

    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{__m256i, _mm256_and_si256, _mm256_loadu_si256, _mm256_storeu_si256};

    const LANES: usize = 4;

    /// AVX2 backend for `dst[i] = lhs[i] & rhs[i]`.
    ///
    /// # Safety
    ///
    /// - Caller must only call this when AVX2 is available.
    /// - `dst` must be valid for writes of `len` initialized `u64` values.
    /// - `lhs` and `rhs` must be valid for reads of `len` initialized `u64` values.
    /// - `dst` must not overlap `lhs` or `rhs`.
    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn and_words(dst: *mut u64, lhs: *const u64, rhs: *const u64, len: usize) {
        let chunks = len / LANES;

        for chunk in 0..chunks {
            let offset = chunk * LANES;

            // SAFETY:
            // - `offset + LANES <= len`.
            // - `_mm256_loadu_si256` and `_mm256_storeu_si256` permit unaligned access.
            // - Pointer validity and non-overlap are guaranteed by the caller.
            unsafe {
                let lhs_vec = _mm256_loadu_si256(lhs.add(offset).cast::<__m256i>());
                let rhs_vec = _mm256_loadu_si256(rhs.add(offset).cast::<__m256i>());
                let out_vec = _mm256_and_si256(lhs_vec, rhs_vec);

                _mm256_storeu_si256(dst.add(offset).cast::<__m256i>(), out_vec);
            }
        }

        let done = chunks * LANES;

        // SAFETY:
        // - `done <= len`.
        // - Tail range is `done..len`.
        // - Pointer validity and non-overlap are guaranteed by the caller.
        unsafe {
            scalar::and_words(dst.add(done), lhs.add(done), rhs.add(done), len - done);
        }
    }
}

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse2 {
    use super::scalar;

    #[cfg(target_arch = "x86")]
    use core::arch::x86::{__m128i, _mm_and_si128, _mm_loadu_si128, _mm_storeu_si128};

    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{__m128i, _mm_and_si128, _mm_loadu_si128, _mm_storeu_si128};

    const LANES: usize = 2;

    /// SSE2 backend for `dst[i] = lhs[i] & rhs[i]`.
    ///
    /// # Safety
    ///
    /// - Caller must only call this when SSE2 is available.
    /// - `dst` must be valid for writes of `len` initialized `u64` values.
    /// - `lhs` and `rhs` must be valid for reads of `len` initialized `u64` values.
    /// - `dst` must not overlap `lhs` or `rhs`.
    #[target_feature(enable = "sse2")]
    pub(super) unsafe fn and_words(dst: *mut u64, lhs: *const u64, rhs: *const u64, len: usize) {
        let chunks = len / LANES;

        for chunk in 0..chunks {
            let offset = chunk * LANES;

            // SAFETY:
            // - `offset + LANES <= len`.
            // - `_mm_loadu_si128` and `_mm_storeu_si128` permit unaligned access.
            // - Pointer validity and non-overlap are guaranteed by the caller.
            unsafe {
                let lhs_vec = _mm_loadu_si128(lhs.add(offset).cast::<__m128i>());
                let rhs_vec = _mm_loadu_si128(rhs.add(offset).cast::<__m128i>());
                let out_vec = _mm_and_si128(lhs_vec, rhs_vec);

                _mm_storeu_si128(dst.add(offset).cast::<__m128i>(), out_vec);
            }
        }

        let done = chunks * LANES;

        // SAFETY:
        // - `done <= len`.
        // - Tail range is `done..len`.
        // - Pointer validity and non-overlap are guaranteed by the caller.
        unsafe {
            scalar::and_words(dst.add(done), lhs.add(done), rhs.add(done), len - done);
        }
    }
}

#[allow(unused)]
#[cfg(target_arch = "aarch64")]
mod neon {
    use super::scalar;
    use core::arch::aarch64::{vandq_u64, vld1q_u64, vst1q_u64};

    const LANES: usize = 2;

    /// NEON backend for `dst[i] = lhs[i] & rhs[i]`.
    ///
    /// # Safety
    ///
    /// - Caller must only call this when NEON is available.
    /// - `dst` must be valid for writes of `len` initialized `u64` values.
    /// - `lhs` and `rhs` must be valid for reads of `len` initialized `u64` values.
    /// - `dst` must not overlap `lhs` or `rhs`.
    #[target_feature(enable = "neon")]
    pub(super) unsafe fn and_words(dst: *mut u64, lhs: *const u64, rhs: *const u64, len: usize) {
        let chunks = len / LANES;

        for chunk in 0..chunks {
            let offset = chunk * LANES;

            // SAFETY:
            // - `offset + LANES <= len`.
            // - Each NEON vector reads/writes 2 u64 values.
            // - Pointer validity and non-overlap are guaranteed by the caller.
            unsafe {
                let lhs_vec = vld1q_u64(lhs.add(offset));
                let rhs_vec = vld1q_u64(rhs.add(offset));
                let out_vec = vandq_u64(lhs_vec, rhs_vec);

                vst1q_u64(dst.add(offset), out_vec);
            }
        }

        let done = chunks * LANES;

        // SAFETY:
        // - `done <= len`.
        // - Tail range is `done..len`.
        // - Pointer validity and non-overlap are guaranteed by the caller.
        unsafe {
            scalar::and_words(dst.add(done), lhs.add(done), rhs.add(done), len - done);
        }
    }
}

#[cfg(test)]
mod tests_for_backend_equivalence;
