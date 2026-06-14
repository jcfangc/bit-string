use alloc::{boxed::Box, vec::Vec};

use crate::bit_string::bits::Bits;

#[inline]
pub(super) fn repeat_core(word_len: usize, value: u64, bit_len: usize) -> Box<[u64]> {
    // Fast path: resize to zero (memset) is cheaper than SIMD fill for zeros.
    if value == 0 {
        let mut out = Vec::<u64>::with_capacity(word_len);
        out.resize(word_len, 0);
        Bits::mask_unused(&mut out, bit_len);
        return out.into_boxed_slice();
    }

    let mut out = Vec::<u64>::with_capacity(word_len);

    // SAFETY:
    // - `out` has capacity for exactly `word_len` u64 values.
    // - `out.as_mut_ptr()` is valid for writes of `word_len` u64 values.
    // - `dispatch` writes every slot in `0..word_len` exactly once.
    unsafe {
        dispatch(out.as_mut_ptr(), word_len, value);
        out.set_len(word_len);
    }

    Bits::mask_unused(&mut out, bit_len);
    out.into_boxed_slice()
}

/// Writes `value` into every slot of `dst[0..word_len]`.
///
/// # Safety
///
/// - `dst` must be valid for writes of `word_len` initialized `u64` values.
#[inline]
unsafe fn dispatch(dst: *mut u64, word_len: usize, value: u64) {
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    {
        // SAFETY:
        // - Forwarded from `dispatch`'s safety contract.
        // - This branch is compiled only when AVX2 is enabled.
        unsafe { avx2::words(dst, word_len, value) };
        return;
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse2",
        not(target_feature = "avx2")
    ))]
    {
        // SAFETY:
        // - Forwarded from `dispatch`'s safety contract.
        // - This branch is compiled only when SSE2 is enabled.
        unsafe { sse2::words(dst, word_len, value) };
        return;
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        // SAFETY:
        // - Forwarded from `dispatch`'s safety contract.
        // - This branch is compiled only when NEON is enabled.
        unsafe { neon::words(dst, word_len, value) };
        return;
    }

    #[allow(unused)]
    // SAFETY: Forwarded from `dispatch`'s safety contract.
    unsafe {
        scalar::words(dst, word_len, value);
    }
}

#[allow(unused)]
mod scalar {
    /// Scalar backend for filling `dst[0..word_len]` with `value`.
    ///
    /// # Safety
    ///
    /// - `dst` must be valid for writes of `word_len` initialized `u64` values.
    #[inline]
    pub(super) unsafe fn words(dst: *mut u64, word_len: usize, value: u64) {
        for i in 0..word_len {
            // SAFETY:
            // - `i < word_len`.
            // - Pointer validity is guaranteed by the caller.
            unsafe {
                dst.add(i).write(value);
            }
        }
    }
}

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2 {
    use super::scalar;

    #[cfg(target_arch = "x86")]
    use core::arch::x86::{__m256i, _mm256_set1_epi64x, _mm256_storeu_si256};

    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{__m256i, _mm256_set1_epi64x, _mm256_storeu_si256};

    const LANES: usize = 4;

    /// AVX2 backend for filling `dst[0..word_len]` with `value`.
    ///
    /// # Safety
    ///
    /// - Caller must only call this when AVX2 is available.
    /// - `dst` must be valid for writes of `word_len` initialized `u64` values.
    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn words(dst: *mut u64, word_len: usize, value: u64) {
        let chunks = word_len / LANES;

        // SAFETY:
        // - This constructor requires AVX2 to be available.
        // - This function is compiled with `target_feature = "avx2"`.
        let fill = _mm256_set1_epi64x(value as i64);

        for chunk in 0..chunks {
            let offset = chunk * LANES;

            // SAFETY:
            // - `offset + LANES <= word_len`.
            // - `_mm256_storeu_si256` permits unaligned writes.
            // - Pointer validity is guaranteed by the caller.
            unsafe {
                _mm256_storeu_si256(dst.add(offset).cast::<__m256i>(), fill);
            }
        }

        let done = chunks * LANES;

        // SAFETY:
        // - `done <= word_len`.
        // - Tail range is `done..word_len`.
        // - Pointer validity is guaranteed by the caller.
        unsafe {
            scalar::words(dst.add(done), word_len - done, value);
        }
    }
}

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse2 {
    use super::scalar;

    #[cfg(target_arch = "x86")]
    use core::arch::x86::{__m128i, _mm_set1_epi64x, _mm_storeu_si128};

    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{__m128i, _mm_set1_epi64x, _mm_storeu_si128};

    const LANES: usize = 2;

    /// SSE2 backend for filling `dst[0..word_len]` with `value`.
    ///
    /// # Safety
    ///
    /// - Caller must only call this when SSE2 is available.
    /// - `dst` must be valid for writes of `word_len` initialized `u64` values.
    #[target_feature(enable = "sse2")]
    pub(super) unsafe fn words(dst: *mut u64, word_len: usize, value: u64) {
        let chunks = word_len / LANES;

        // SAFETY:
        // - This constructor requires SSE2 to be available.
        // - This function is compiled with `target_feature = "sse2"`.
        let fill = _mm_set1_epi64x(value as i64);

        for chunk in 0..chunks {
            let offset = chunk * LANES;

            // SAFETY:
            // - `offset + LANES <= word_len`.
            // - `_mm_storeu_si128` permits unaligned writes.
            // - Pointer validity is guaranteed by the caller.
            unsafe {
                _mm_storeu_si128(dst.add(offset).cast::<__m128i>(), fill);
            }
        }

        let done = chunks * LANES;

        // SAFETY:
        // - `done <= word_len`.
        // - Tail range is `done..word_len`.
        // - Pointer validity is guaranteed by the caller.
        unsafe {
            scalar::words(dst.add(done), word_len - done, value);
        }
    }
}

#[allow(unused)]
#[cfg(target_arch = "aarch64")]
mod neon {
    use super::scalar;

    use core::arch::aarch64::{vdupq_n_u64, vst1q_u64};

    const LANES: usize = 2;

    /// NEON backend for filling `dst[0..word_len]` with `value`.
    ///
    /// # Safety
    ///
    /// - Caller must only call this when NEON is available.
    /// - `dst` must be valid for writes of `word_len` initialized `u64` values.
    #[target_feature(enable = "neon")]
    pub(super) unsafe fn words(dst: *mut u64, word_len: usize, value: u64) {
        let chunks = word_len / LANES;

        // SAFETY:
        // - This constructor requires NEON to be available.
        // - This function is compiled with `target_feature = "neon"`.
        let fill = vdupq_n_u64(value);

        for chunk in 0..chunks {
            let offset = chunk * LANES;

            // SAFETY:
            // - `offset + LANES <= word_len`.
            // - Each NEON vector writes 2 u64 values.
            // - Pointer validity is guaranteed by the caller.
            unsafe {
                vst1q_u64(dst.add(offset), fill);
            }
        }

        let done = chunks * LANES;

        // SAFETY:
        // - `done <= word_len`.
        // - Tail range is `done..word_len`.
        // - Pointer validity is guaranteed by the caller.
        unsafe {
            scalar::words(dst.add(done), word_len - done, value);
        }
    }
}

#[cfg(test)]
mod tests_for_backend_equivalence;
