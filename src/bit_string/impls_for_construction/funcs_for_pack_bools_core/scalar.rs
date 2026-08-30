use super::scalar;

/// Scalar backend: accumulate 64 bytes at a time into one u64.
///
/// # Safety
///
/// - `src` must be valid for reads of `bit_len` u8 values.
/// - `dst` must be valid for writes of `ceil(bit_len / 64)` u64 values.
#[inline]
pub(super) unsafe fn words(mut dst: *mut u64, mut src: *const u8, mut bit_len: usize) {
    while bit_len >= 64 {
        // SAFETY: `bit_len >= 64`, so reading 64 bytes from `src` is valid.
        // `dst` points to the next output slot.
        unsafe {
            *dst = scalar::pack_64(src);
        }
        // SAFETY:
        // - `dst` advances by 1 word; the caller ensures the destination
        //   has enough capacity for all full-word writes.
        // - `src` advances by 64 bytes; `bit_len >= 64` ensures read bounds.
        unsafe {
            dst = dst.add(1);
            src = src.add(64);
        }
        bit_len -= 64;
    }

    if bit_len > 0 {
        // SAFETY: `bit_len > 0`, so reading `bit_len` bytes from `src` is valid.
        unsafe {
            *dst = scalar::pack_partial(src, bit_len);
        }
    }
}

/// Pack exactly 64 bytes into one u64 (little-endian: byte i → bit i).
///
/// # Safety
///
/// `src` must be valid for reads of 64 u8 values.
#[inline]
unsafe fn pack_64(src: *const u8) -> u64 {
    let mut word = 0u64;
    for i in 0..64 {
        // SAFETY:
        // - `i < 64`, offset is in bounds.
        // - `src` is valid for 64 reads per caller contract.
        let byte = unsafe { src.add(i).read() };
        word |= ((byte & 1) as u64) << i;
    }
    word
}

/// Pack fewer than 64 bytes into one u64.
///
/// # Safety
///
/// `src` must be valid for reads of `len` u8 values (`len < 64`).
#[inline]
unsafe fn pack_partial(src: *const u8, len: usize) -> u64 {
    let mut word = 0u64;
    for i in 0..len {
        // SAFETY:
        // - `i < len < 64`.
        // - `src` is valid for `len` reads per caller contract.
        let byte = unsafe { src.add(i).read() };
        word |= ((byte & 1) as u64) << i;
    }
    word
}
