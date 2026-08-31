use alloc::vec::Vec;

use crate::traits::*;
use crate::word_len;

/// Pack `bit_len` LSBs from `src` into a `Vec<u64>`.
///
/// Each source byte is treated as one bit (0 → 0, non-zero → 1).
/// Bits are packed in little-endian order: byte `i` becomes bit `i % 64`
/// of word `i / 64`.
#[inline]
pub(super) fn bools_core(src: *const u8, bit_len: usize) -> Vec<u64> {
    let word_len = word_len(bit_len);
    let mut out = Vec::<u64>::with_capacity(word_len);

    // SAFETY:
    // - `out` has capacity for exactly `word_len` u64 values.
    // - `out.as_mut_ptr()` is valid for writes of `word_len` u64 values.
    // - `dispatch` writes every slot in `0..word_len` exactly once.
    unsafe {
        dispatch(out.as_mut_ptr(), src, bit_len);
        out.set_len(word_len);
    }

    out.mask_unused_bits(bit_len);
    out
}

/// Packs `bit_len` bytes from `src` into u64 words at `dst`.
///
/// # Safety
///
/// - `src` must be valid for reads of `bit_len` u8 values.
/// - `dst` must be valid for writes of `ceil(bit_len / 64)` u64 values.
#[inline]
unsafe fn dispatch(dst: *mut u64, src: *const u8, bit_len: usize) {
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    {
        // SAFETY: caller guarantees `dst`/`src` pointer validity and word count. AVX2 availability was confirmed by `#[target_feature]`.
        unsafe { avx2::words(dst, src, bit_len) };
        return;
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse2",
        not(target_feature = "avx2")
    ))]
    {
        // SAFETY: caller guarantees `dst`/`src` pointer validity and word count. SSE2 availability was confirmed by `#[target_feature]`.
        unsafe { sse2::words(dst, src, bit_len) };
        return;
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        // SAFETY: caller guarantees `dst`/`src` pointer validity and word count. NEON availability was confirmed by `#[target_feature]`.
        unsafe { neon::words(dst, src, bit_len) };
        return;
    }

    // SAFETY: caller guarantees pointer validity. Scalar backend is always safe.
    #[allow(unused)]
    unsafe {
        scalar::words(dst, src, bit_len);
    }
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
