use super::scalar;

use core::arch::aarch64::{vaddvq_u8, vcntq_u8, vld1q_u64, vreinterpretq_u8_u64};

const LANES: usize = 2;

/// NEON backend for counting set bits in `src[0..len]`.
///
/// Counts bits per byte with `vcntq_u8`, then horizontally sums byte counts.
///
/// # Safety
///
/// - Caller must only call this when NEON is available.
/// - `src` must be valid for reads of `len` initialized `u64` values.
#[target_feature(enable = "neon")]
pub(super) unsafe fn count_words(src: *const u64, len: usize) -> usize {
    let chunks = len / LANES;
    let mut count = 0usize;

    for chunk in 0..chunks {
        let offset = chunk * LANES;

        // SAFETY:
        // - `offset + LANES <= len`.
        // - `vld1q_u64` reads exactly 2 u64 values.
        // - `src` validity is guaranteed by the caller.
        unsafe {
            let words = vld1q_u64(src.add(offset));
            let bytes = vreinterpretq_u8_u64(words);
            let byte_counts = vcntq_u8(bytes);

            count += vaddvq_u8(byte_counts) as usize;
        }
    }

    let done = chunks * LANES;

    // SAFETY:
    // - `done <= len`.
    // - Tail range is `done..len`.
    // - Pointer validity is guaranteed by the caller.
    count + unsafe { scalar::count_words(src.add(done), len - done) }
}
