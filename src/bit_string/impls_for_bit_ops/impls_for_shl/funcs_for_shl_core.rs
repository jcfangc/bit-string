use alloc::vec::Vec;

use crate::WORD_BITS;

use crate::bit_string::bits::*;

#[inline]
pub(super) fn owned(src: &[u64], bit_len: usize, amount: usize) -> Vec<u64> {
    let word_len = src.len();
    let mut out = Vec::<u64>::with_capacity(word_len);

    // SAFETY:
    // - `out` has capacity for exactly `word_len` u64 values.
    // - `src` is valid for reads of `word_len` u64 values.
    // - `out.as_mut_ptr()` is valid for writes of `word_len` u64 values.
    // - `out` is freshly allocated, so it cannot overlap `src`.
    // - `dispatch` writes every slot in `0..word_len` exactly once.
    unsafe {
        dispatch(out.as_mut_ptr(), src.as_ptr(), word_len, amount);
        out.set_len(word_len);
    }

    out.mask_unused_bits(bit_len);
    out
}

#[inline]
pub(super) fn assign(bits: &mut [u64], bit_len: usize, amount: usize) {
    let word_len = bits.len();
    let ptr = bits.as_mut_ptr();

    // SAFETY:
    // - `ptr` is valid for reads and writes of `word_len` u64 values.
    // - `dst == src` is explicitly allowed by `dispatch`.
    // - `dispatch` writes every slot in `0..word_len` exactly once.
    // - The implementation writes from high word to low word, so in-place left shift
    //   does not overwrite source words before they are read.
    unsafe {
        dispatch(ptr, ptr.cast_const(), word_len, amount);
    }

    bits.mask_unused_bits(bit_len);
}

/// Writes `src << amount` into `dst`.
///
/// This is a word-level left shift. The caller is responsible for masking unused
/// bits in the final `BitString` word.
///
/// `dst` may be exactly equal to `src`, which enables in-place assignment.
/// Partial overlaps are not allowed.
///
/// # Safety
///
/// - `dst` must be valid for writes of `word_len` initialized `u64` values.
/// - `src` must be valid for reads of `word_len` initialized `u64` values.
/// - `dst` must either:
///   - not overlap `src`, or
///   - be exactly equal to `src`.
#[inline]
unsafe fn dispatch(dst: *mut u64, src: *const u64, word_len: usize, amount: usize) {
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    {
        // SAFETY:
        // - Forwarded from `dispatch`'s safety contract.
        // - This branch is compiled only when AVX2 is enabled.
        unsafe { avx2::words(dst, src, word_len, amount) };
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
        unsafe { sse2::words(dst, src, word_len, amount) };
        return;
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        // SAFETY:
        // - Forwarded from `dispatch`'s safety contract.
        // - This branch is compiled only when NEON is enabled.
        unsafe { neon::words(dst, src, word_len, amount) };
        return;
    }

    #[allow(unused)]
    // SAFETY: Forwarded from `dispatch`'s safety contract.
    unsafe {
        scalar::words(dst, src, word_len, amount);
    }
}

#[inline]
fn split_amount(amount: usize) -> (usize, usize) {
    (amount / WORD_BITS, amount % WORD_BITS)
}

#[inline]
unsafe fn scalar_word(src: *const u64, word_len: usize, dst_index: usize, amount: usize) -> u64 {
    let (word_shift, bit_shift) = split_amount(amount);

    let Some(src_index) = dst_index.checked_sub(word_shift) else {
        return 0;
    };

    if src_index >= word_len {
        return 0;
    }

    // SAFETY:
    // - `src_index < word_len`.
    // - Pointer validity is guaranteed by the caller.
    let mut out = unsafe { src.add(src_index).read() << bit_shift };

    if bit_shift != 0 && src_index > 0 {
        // SAFETY:
        // - `src_index > 0`, so `src_index - 1 < word_len`.
        // - Pointer validity is guaranteed by the caller.
        out |= unsafe { src.add(src_index - 1).read() >> (WORD_BITS - bit_shift) };
    }

    out
}

#[allow(unused)]
mod scalar {
    use super::scalar_word;

    /// Scalar backend for word-level left shift.
    ///
    /// Supports `dst == src`.
    ///
    /// # Safety
    ///
    /// - `dst` must be valid for writes of `word_len` initialized `u64` values.
    /// - `src` must be valid for reads of `word_len` initialized `u64` values.
    /// - `dst` must either not overlap `src`, or be exactly equal to `src`.
    pub(super) unsafe fn words(dst: *mut u64, src: *const u64, word_len: usize, amount: usize) {
        for dst_index in (0..word_len).rev() {
            // SAFETY:
            // - `dst_index < word_len`.
            // - Pointer validity and overlap constraints are guaranteed by the caller.
            // - Descending order makes `dst == src` safe for left shift.
            unsafe {
                let word = scalar_word(src, word_len, dst_index, amount);
                dst.add(dst_index).write(word);
            }
        }
    }
}

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2 {
    use super::{scalar_word, split_amount};
    use crate::WORD_BITS;

    #[cfg(target_arch = "x86")]
    use core::arch::x86::{
        __m128i, __m256i, _mm_cvtsi64_si128, _mm256_loadu_si256, _mm256_or_si256, _mm256_sll_epi64,
        _mm256_srl_epi64, _mm256_storeu_si256,
    };

    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m128i, __m256i, _mm_cvtsi64_si128, _mm256_loadu_si256, _mm256_or_si256, _mm256_sll_epi64,
        _mm256_srl_epi64, _mm256_storeu_si256,
    };

    const LANES: usize = 4;

    /// AVX2 backend for word-level left shift.
    ///
    /// Supports `dst == src`.
    ///
    /// # Safety
    ///
    /// - Caller must only call this when AVX2 is available.
    /// - `dst` must be valid for writes of `word_len` initialized `u64` values.
    /// - `src` must be valid for reads of `word_len` initialized `u64` values.
    /// - `dst` must either not overlap `src`, or be exactly equal to `src`.
    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn words(dst: *mut u64, src: *const u64, word_len: usize, amount: usize) {
        let (word_shift, bit_shift) = split_amount(amount);

        if word_shift >= word_len {
            for dst_index in (0..word_len).rev() {
                // SAFETY:
                // - `dst_index < word_len`.
                // - Pointer validity is guaranteed by the caller.
                unsafe { dst.add(dst_index).write(0) };
            }
            return;
        }

        let bulk_start = word_shift + usize::from(bit_shift != 0);
        let mut end = word_len;

        // Process the vectorizable suffix from high to low.
        while end >= bulk_start + LANES {
            let dst_start = end - LANES;
            let src_start = dst_start - word_shift;

            // SAFETY:
            // - `dst_start + LANES <= word_len`.
            // - `src_start + LANES <= word_len`.
            // - If `bit_shift != 0`, `bulk_start` guarantees `src_start > 0`.
            // - Unaligned load/store intrinsics permit unaligned access.
            // - `dst == src` is safe because all loads happen before the store,
            //   and chunks are processed from high to low.
            unsafe {
                let cur = _mm256_loadu_si256(src.add(src_start).cast::<__m256i>());
                let out = if bit_shift == 0 {
                    cur
                } else {
                    let prev = _mm256_loadu_si256(src.add(src_start - 1).cast::<__m256i>());
                    let left_count: __m128i = _mm_cvtsi64_si128(bit_shift as i64);
                    let right_count: __m128i = _mm_cvtsi64_si128((WORD_BITS - bit_shift) as i64);

                    let left = _mm256_sll_epi64(cur, left_count);
                    let right = _mm256_srl_epi64(prev, right_count);

                    _mm256_or_si256(left, right)
                };

                _mm256_storeu_si256(dst.add(dst_start).cast::<__m256i>(), out);
            }

            end = dst_start;
        }

        // Scalar tail below the vectorized suffix, still high to low.
        for dst_index in (0..end).rev() {
            // SAFETY:
            // - `dst_index < word_len`.
            // - Pointer validity and overlap constraints are guaranteed by the caller.
            unsafe {
                let word = scalar_word(src, word_len, dst_index, amount);
                dst.add(dst_index).write(word);
            }
        }
    }
}

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse2 {
    use super::{scalar_word, split_amount};
    use crate::WORD_BITS;

    #[cfg(target_arch = "x86")]
    use core::arch::x86::{
        __m128i, _mm_cvtsi64_si128, _mm_loadu_si128, _mm_or_si128, _mm_sll_epi64, _mm_srl_epi64,
        _mm_storeu_si128,
    };

    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{
        __m128i, _mm_cvtsi64_si128, _mm_loadu_si128, _mm_or_si128, _mm_sll_epi64, _mm_srl_epi64,
        _mm_storeu_si128,
    };

    const LANES: usize = 2;

    /// SSE2 backend for word-level left shift.
    ///
    /// Supports `dst == src`.
    ///
    /// # Safety
    ///
    /// - Caller must only call this when SSE2 is available.
    /// - `dst` must be valid for writes of `word_len` initialized `u64` values.
    /// - `src` must be valid for reads of `word_len` initialized `u64` values.
    /// - `dst` must either not overlap `src`, or be exactly equal to `src`.
    #[target_feature(enable = "sse2")]
    pub(super) unsafe fn words(dst: *mut u64, src: *const u64, word_len: usize, amount: usize) {
        let (word_shift, bit_shift) = split_amount(amount);

        if word_shift >= word_len {
            for dst_index in (0..word_len).rev() {
                // SAFETY:
                // - `dst_index < word_len`.
                // - Pointer validity is guaranteed by the caller.
                unsafe { dst.add(dst_index).write(0) };
            }
            return;
        }

        let bulk_start = word_shift + usize::from(bit_shift != 0);
        let mut end = word_len;

        while end >= bulk_start + LANES {
            let dst_start = end - LANES;
            let src_start = dst_start - word_shift;

            // SAFETY:
            // - `dst_start + LANES <= word_len`.
            // - `src_start + LANES <= word_len`.
            // - If `bit_shift != 0`, `bulk_start` guarantees `src_start > 0`.
            // - Unaligned load/store intrinsics permit unaligned access.
            // - `dst == src` is safe because all loads happen before the store,
            //   and chunks are processed from high to low.
            unsafe {
                let cur = _mm_loadu_si128(src.add(src_start).cast::<__m128i>());
                let out = if bit_shift == 0 {
                    cur
                } else {
                    let prev = _mm_loadu_si128(src.add(src_start - 1).cast::<__m128i>());
                    let left_count = _mm_cvtsi64_si128(bit_shift as i64);
                    let right_count = _mm_cvtsi64_si128((WORD_BITS - bit_shift) as i64);

                    let left = _mm_sll_epi64(cur, left_count);
                    let right = _mm_srl_epi64(prev, right_count);

                    _mm_or_si128(left, right)
                };

                _mm_storeu_si128(dst.add(dst_start).cast::<__m128i>(), out);
            }

            end = dst_start;
        }

        for dst_index in (0..end).rev() {
            // SAFETY:
            // - `dst_index < word_len`.
            // - Pointer validity and overlap constraints are guaranteed by the caller.
            unsafe {
                let word = scalar_word(src, word_len, dst_index, amount);
                dst.add(dst_index).write(word);
            }
        }
    }
}

#[allow(unused)]
#[cfg(target_arch = "aarch64")]
mod neon {
    use super::{scalar_word, split_amount};
    use crate::WORD_BITS;

    use core::arch::aarch64::{
        int64x2_t, uint64x2_t, vdupq_n_s64, vld1q_u64, vorrq_u64, vshlq_u64, vst1q_u64,
    };

    const LANES: usize = 2;

    #[inline]
    #[target_feature(enable = "neon")]
    unsafe fn shl_vec(src: uint64x2_t, amount: usize) -> uint64x2_t {
        // SAFETY:
        // - This helper is called only from `words`, which enables NEON.
        unsafe { vshlq_u64(src, vdupq_n_s64(amount as i64) as int64x2_t) }
    }

    #[inline]
    #[target_feature(enable = "neon")]
    unsafe fn shr_vec(src: uint64x2_t, amount: usize) -> uint64x2_t {
        // SAFETY:
        // - This helper is called only from `words`, which enables NEON.
        unsafe { vshlq_u64(src, vdupq_n_s64(-(amount as i64)) as int64x2_t) }
    }

    #[inline]
    #[target_feature(enable = "neon")]
    unsafe fn or_vec(lhs: uint64x2_t, rhs: uint64x2_t) -> uint64x2_t {
        // SAFETY:
        // - This helper is called only from `words`, which enables NEON.
        unsafe { vorrq_u64(lhs, rhs) }
    }

    /// NEON backend for word-level left shift.
    ///
    /// Supports `dst == src`.
    ///
    /// # Safety
    ///
    /// - Caller must only call this when NEON is available.
    /// - `dst` must be valid for writes of `word_len` initialized `u64` values.
    /// - `src` must be valid for reads of `word_len` initialized `u64` values.
    /// - `dst` must either not overlap `src`, or be exactly equal to `src`.
    #[target_feature(enable = "neon")]
    pub(super) unsafe fn words(dst: *mut u64, src: *const u64, word_len: usize, amount: usize) {
        let (word_shift, bit_shift) = split_amount(amount);

        if word_shift >= word_len {
            for dst_index in (0..word_len).rev() {
                // SAFETY:
                // - `dst_index < word_len`.
                // - Pointer validity is guaranteed by the caller.
                unsafe { dst.add(dst_index).write(0) };
            }
            return;
        }

        let bulk_start = word_shift + usize::from(bit_shift != 0);
        let mut end = word_len;

        while end >= bulk_start + LANES {
            let dst_start = end - LANES;
            let src_start = dst_start - word_shift;

            // SAFETY:
            // - `dst_start + LANES <= word_len`.
            // - `src_start + LANES <= word_len`.
            // - If `bit_shift != 0`, `bulk_start` guarantees `src_start > 0`.
            // - `vld1q_u64` reads exactly 2 u64 values.
            // - `vst1q_u64` writes exactly 2 u64 values.
            // - `dst == src` is safe because all loads happen before the store,
            //   and chunks are processed from high to low.
            unsafe {
                let cur = vld1q_u64(src.add(src_start));
                let out = if bit_shift == 0 {
                    cur
                } else {
                    let prev = vld1q_u64(src.add(src_start - 1));
                    let left = shl_vec(cur, bit_shift);
                    let right = shr_vec(prev, WORD_BITS - bit_shift);

                    or_vec(left, right)
                };

                vst1q_u64(dst.add(dst_start), out);
            }

            end = dst_start;
        }

        for dst_index in (0..end).rev() {
            // SAFETY:
            // - `dst_index < word_len`.
            // - Pointer validity and overlap constraints are guaranteed by the caller.
            unsafe {
                let word = scalar_word(src, word_len, dst_index, amount);
                dst.add(dst_index).write(word);
            }
        }
    }
}

#[cfg(test)]
mod tests_for_backend_equivalence;
