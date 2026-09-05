use super::*;

#[inline]
fn apply<const OP: u8>(lhs: u64, rhs: u64) -> u64 {
    match OP {
        OP_AND => lhs & rhs,
        OP_OR => lhs | rhs,
        OP_XOR => lhs ^ rhs,
        _ => unreachable!("unsupported binary bit operation"),
    }
}

/// Scalar backend for `dst[i] = lhs[i] OP rhs[i]`.
///
/// Supports `dst == lhs`.
///
/// # Safety
///
/// - `dst` must be valid for writes of `len` initialized `u64` values.
/// - `lhs` and `rhs` must be valid for reads of `len` initialized `u64` values.
/// - `dst` must either not overlap `lhs`, or be exactly equal to `lhs`.
/// - `dst` must not overlap `rhs`.
#[inline]
pub(super) unsafe fn words<const OP: u8>(
    dst: *mut u64,
    lhs: *const u64,
    rhs: *const u64,
    len: usize,
) {
    for i in 0..len {
        // SAFETY:
        // - `i < len`.
        // - Pointer validity and overlap constraints are guaranteed by the caller.
        // - `dst == lhs` is safe because both operands are read before writing `dst[i]`.
        unsafe {
            let lhs_word = lhs.add(i).read();
            let rhs_word = rhs.add(i).read();
            dst.add(i).write(apply::<OP>(lhs_word, rhs_word));
        }
    }
}
