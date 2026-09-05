use alloc::vec::Vec;

use crate::traits::*;

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
        scalar::words(dst, src, len)
    };
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
