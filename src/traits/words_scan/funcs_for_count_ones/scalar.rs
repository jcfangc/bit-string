/// Scalar backend for counting set bits in `src[0..len]`.
///
/// # Safety
///
/// - `src` must be valid for reads of `len` initialized `u64` values.
#[inline]
pub(super) unsafe fn count_words(src: *const u64, len: usize) -> usize {
    let mut count = 0usize;

    for i in 0..len {
        // SAFETY:
        // - `i < len`.
        // - Pointer validity is guaranteed by the caller.
        unsafe {
            count += src.add(i).read().count_ones() as usize;
        }
    }

    count
}
