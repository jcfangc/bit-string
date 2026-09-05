use alloc::vec::Vec;

use crate::traits::*;
use crate::word_len;

/// Pack `bit_len` ASCII '0'/'1' bytes from `src` into a `Vec<u64>`.
///
/// Returns `Err((index, byte))` on the first invalid character.
/// Bits are packed in little-endian order: byte `i` becomes bit `i % 64`
/// of word `i / 64`.
#[inline]
pub(super) fn str_core(src: *const u8, bit_len: usize) -> Result<Vec<u64>, (usize, u8)> {
    let word_len = word_len(bit_len);
    let mut out = Vec::<u64>::with_capacity(word_len);

    // SAFETY:
    // - `out` has capacity for `word_len` u64 values.
    // - `out.as_mut_ptr()` is valid for writes of up to `word_len` u64 values.
    // - `dispatch` either writes all slots or returns an error (in which case
    //   the Vec is dropped without reading uninitialized memory).
    let error = unsafe { dispatch(out.as_mut_ptr(), src, bit_len) };

    if let Some((idx, byte)) = error {
        return Err((idx, byte));
    }

    // SAFETY: `dispatch` returned `None`, meaning it successfully wrote
    // every slot in `0..word_len`.
    unsafe { out.set_len(word_len) };

    out.mask_unused_bits(bit_len);
    Ok(out)
}

/// Validates and packs `bit_len` ASCII '0'/'1' bytes.
///
/// Returns `None` on success, or `Some((index, invalid_byte))` on the first
/// byte that is neither b'0' (0x30) nor b'1' (0x31).
///
/// # Safety
///
/// - `src` must be valid for reads of `bit_len` u8 values.
/// - `dst` must be valid for writes of `ceil(bit_len / 64)` u64 values.
#[inline]
unsafe fn dispatch(dst: *mut u64, src: *const u8, bit_len: usize) -> Option<(usize, u8)> {
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    {
        // SAFETY: caller guarantees `dst`/`src` pointer validity and byte count.
        // AVX2 is guaranteed by `#[cfg(target_feature = "avx2")]`.
        return unsafe { avx2::words(dst, src, bit_len) };
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse2",
        not(target_feature = "avx2")
    ))]
    {
        // SAFETY: caller guarantees `dst`/`src` pointer validity and byte count.
        // SSE2 is guaranteed by `#[cfg(target_feature = "sse2")]`.
        return unsafe { sse2::words(dst, src, bit_len) };
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        // SAFETY: caller guarantees `dst`/`src` pointer validity and byte count.
        // NEON is guaranteed by `#[cfg(target_feature = "neon")]`.
        return unsafe { neon::words(dst, src, bit_len) };
    }

    #[allow(unused)]
    // SAFETY: caller guarantees pointer validity. Scalar backend is always safe.
    unsafe {
        scalar::words(dst, src, bit_len)
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
