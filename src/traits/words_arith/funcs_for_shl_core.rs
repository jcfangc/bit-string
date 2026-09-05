use alloc::vec::Vec;

use crate::WORD_BITS;

use crate::traits::*;

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
        scalar::words(dst, src, word_len, amount)
    };
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
mod scalar;

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2;

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse2;

#[allow(unused)]
#[cfg(target_arch = "aarch64")]
mod neon;

#[cfg(test)]
mod tests_for_backend_equivalence;
