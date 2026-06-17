use alloc::vec::Vec;

use crate::bit_string::bits::*;

#[inline]
pub(super) fn owned(src: &[u64], bit_len: usize) -> Vec<u64> {
    let word_len = src.len();
    let mut out = Vec::<u64>::with_capacity(word_len);

    // SAFETY:
    // - `out` has capacity for exactly `word_len` u64 values.
    // - `src` is valid for reads of `word_len` u64 values.
    // - `out.as_mut_ptr()` is valid for writes of `word_len` u64 values.
    // - `out` is freshly allocated, so it cannot overlap `src`.
    // - `dispatch` writes every slot in `0..word_len` exactly once.
    unsafe {
        dispatch(out.as_mut_ptr(), src.as_ptr(), word_len);
        out.set_len(word_len);
    }

    out.mask_unused_bits(bit_len);
    out
}

#[inline]
pub(super) fn assign(bits: &mut [u64], bit_len: usize) {
    let word_len = bits.len();
    let ptr = bits.as_mut_ptr();

    // SAFETY:
    // - `ptr` is valid for reads and writes of `word_len` u64 values.
    // - `dst == src` is explicitly allowed by `dispatch`.
    // - `dispatch` writes every slot in `0..word_len` exactly once.
    unsafe {
        dispatch(ptr, ptr.cast_const(), word_len);
    }

    bits.mask_unused_bits(bit_len);
}

/// Writes `!src[i]` into `dst[i]` for every `i in 0..len`.
///
/// `dst` may be exactly equal to `src`, which enables in-place assignment.
/// Partial overlaps are not allowed.
///
/// # Safety
///
/// - `dst` must be valid for writes of `len` initialized `u64` values.
/// - `src` must be valid for reads of `len` initialized `u64` values.
/// - `dst` must either:
///   - not overlap `src`, or
///   - be exactly equal to `src`.
#[inline]
unsafe fn dispatch(dst: *mut u64, src: *const u64, len: usize) {
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    {
        // SAFETY:
        // - Forwarded from `dispatch`'s safety contract.
        // - This branch is compiled only when AVX2 is enabled.
        unsafe { avx2::words(dst, src, len) };
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
        unsafe { sse2::words(dst, src, len) };
        return;
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        // SAFETY:
        // - Forwarded from `dispatch`'s safety contract.
        // - This branch is compiled only when NEON is enabled.
        unsafe { neon::words(dst, src, len) };
        return;
    }

    #[allow(unused)]
    // SAFETY: Forwarded from `dispatch`'s safety contract.
    unsafe {
        scalar::words(dst, src, len);
    }
}

#[allow(unused)]
mod scalar {
    /// Scalar backend for `dst[i] = !src[i]`.
    ///
    /// Supports `dst == src`.
    ///
    /// # Safety
    ///
    /// - `dst` must be valid for writes of `len` initialized `u64` values.
    /// - `src` must be valid for reads of `len` initialized `u64` values.
    /// - `dst` must either not overlap `src`, or be exactly equal to `src`.
    #[inline]
    pub(super) unsafe fn words(dst: *mut u64, src: *const u64, len: usize) {
        for i in 0..len {
            // SAFETY:
            // - `i < len`.
            // - Pointer validity and overlap constraints are guaranteed by the caller.
            // - `dst == src` is safe because `src[i]` is read before `dst[i]` is written.
            unsafe {
                let word = src.add(i).read();
                dst.add(i).write(!word);
            }
        }
    }
}

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2 {
    use super::scalar;

    #[cfg(target_arch = "x86")]
    use core::arch::x86::{
        __m256i, _mm256_cmpeq_epi8, _mm256_loadu_si256, _mm256_setzero_si256, _mm256_storeu_si256,
        _mm256_xor_si256,
    };

    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m256i, _mm256_cmpeq_epi8, _mm256_loadu_si256, _mm256_setzero_si256, _mm256_storeu_si256,
        _mm256_xor_si256,
    };

    const LANES: usize = 4;

    /// AVX2 backend for `dst[i] = !src[i]`.
    ///
    /// Supports `dst == src`.
    ///
    /// # Safety
    ///
    /// - Caller must only call this when AVX2 is available.
    /// - `dst` must be valid for writes of `len` initialized `u64` values.
    /// - `src` must be valid for reads of `len` initialized `u64` values.
    /// - `dst` must either not overlap `src`, or be exactly equal to `src`.
    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn words(dst: *mut u64, src: *const u64, len: usize) {
        let chunks = len / LANES;

        for chunk in 0..chunks {
            let offset = chunk * LANES;

            // SAFETY:
            // - `offset + LANES <= len`.
            // - `_mm256_loadu_si256` and `_mm256_storeu_si256` permit unaligned access.
            // - Pointer validity and overlap constraints are guaranteed by the caller.
            // - `dst == src` is safe because load happens before store.
            unsafe {
                let zero = _mm256_setzero_si256();
                let all_ones = _mm256_cmpeq_epi8(zero, zero);
                let src_vec = _mm256_loadu_si256(src.add(offset).cast::<__m256i>());
                let out_vec = _mm256_xor_si256(src_vec, all_ones);

                _mm256_storeu_si256(dst.add(offset).cast::<__m256i>(), out_vec);
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
}

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse2 {
    use super::scalar;

    #[cfg(target_arch = "x86")]
    use core::arch::x86::{
        __m128i, _mm_cmpeq_epi8, _mm_loadu_si128, _mm_setzero_si128, _mm_storeu_si128,
        _mm_xor_si128,
    };

    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m128i, _mm_cmpeq_epi8, _mm_loadu_si128, _mm_setzero_si128, _mm_storeu_si128,
        _mm_xor_si128,
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
}

#[allow(unused)]
#[cfg(target_arch = "aarch64")]
mod neon {
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
}

#[cfg(test)]
mod tests_for_backend_equivalence;
