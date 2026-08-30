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
