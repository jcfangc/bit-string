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

    // In-place carry stream, inspired by grit-bitvec's insert.
    // A sub-word shift needs only the current old word and its preceding carry.
    // Save that carry before overwriting the word, so low-to-high is safe.
    if dst.cast_const() == src && word_shift == 0 && bit_shift != 0 {
        // SAFETY: dst spans word_len initialized words; end is one-past that
        // same allocation. The cursor advances only while it is below end.
        // Exact overlap means each old word is read before it is overwritten;
        // its outgoing carry is retained before advancing to the next word.
        // At word i, carry is old[i - 1] >> (64 - bit_shift), or zero for i=0.
        // Thus the write produces exactly the shifted word without rereading
        // overwritten input. This branch guarantees 1 <= bit_shift < 64.
        unsafe {
            let end = dst.add(word_len);
            let mut cursor = dst;
            let mut carry = 0;
            while cursor < end {
                let old = cursor.read();
                let next_carry = old >> (WORD_BITS - bit_shift);
                cursor.write((old << bit_shift) | carry);
                cursor = cursor.add(1);
                carry = next_carry;
            }
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
