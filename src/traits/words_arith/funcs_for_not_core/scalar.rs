/// Scalar backend for `dst[i] = !src[i]`.
///
/// Supports `dst == src`.
///
/// # Safety
///
/// - `dst` must be valid for writes of `len` initialized `u64` values.
/// - `src` must be valid for reads of `len` initialized `u64` values.
/// - `dst` must either not overlap `src`, or be exactly equal to `src`.
#[inline]
pub(super) unsafe fn words(dst: *mut u64, src: *const u64, len: usize) {
    for i in 0..len {
        // SAFETY:
        // - `i < len`.
        // - Pointer validity and overlap constraints are guaranteed by the caller.
        // - `dst == src` is safe because `src[i]` is read before `dst[i]` is written.
        unsafe {
            let word = src.add(i).read();
            dst.add(i).write(!word);
        }
    }
}
