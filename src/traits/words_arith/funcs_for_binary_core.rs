use alloc::vec::Vec;

pub(super) const OP_AND: u8 = 0;
pub(super) const OP_OR: u8 = 1;
pub(super) const OP_XOR: u8 = 2;

#[inline]
pub(super) fn owned<const OP: u8>(lhs: &[u64], rhs: &[u64]) -> Vec<u64> {
    debug_assert_eq!(lhs.len(), rhs.len());

    let len = lhs.len();
    let mut out = Vec::<u64>::with_capacity(len);

    // SAFETY:
    // - `out` has capacity for exactly `len` u64 values.
    // - `lhs` and `rhs` are valid for reads of `len` u64 values.
    // - `out.as_mut_ptr()` is valid for writes of `len` u64 values.
    // - `out` is freshly allocated, so it cannot overlap `lhs` or `rhs`.
    // - `dispatch` writes every slot in `0..len` exactly once.
    unsafe {
        dispatch::<OP>(out.as_mut_ptr(), lhs.as_ptr(), rhs.as_ptr(), len);
        out.set_len(len);
    }

    out
}

#[inline]
pub(super) fn assign<const OP: u8>(lhs: &mut [u64], rhs: &[u64]) {
    debug_assert_eq!(lhs.len(), rhs.len());

    let len = lhs.len();
    let lhs_ptr = lhs.as_mut_ptr();

    // SAFETY:
    // - `lhs_ptr` is valid for reads and writes of `len` u64 values.
    // - `rhs` is valid for reads of `len` u64 values.
    // - `dst == lhs` is explicitly allowed by `dispatch`.
    // - Safe Rust prevents `rhs` from aliasing `lhs` in normal calls.
    // - `dispatch` writes every slot in `0..len` exactly once.
    unsafe {
        dispatch::<OP>(lhs_ptr, lhs_ptr.cast_const(), rhs.as_ptr(), len);
    }
}

/// Writes `lhs[i] OP rhs[i]` into `dst[i]` for every `i in 0..len`.
///
/// `dst` may be exactly equal to `lhs`, which enables in-place assignment.
/// Partial overlaps are not allowed.
///
/// # Safety
///
/// - `dst` must be valid for writes of `len` initialized `u64` values.
/// - `lhs` and `rhs` must be valid for reads of `len` initialized `u64` values.
/// - `dst` must either not overlap `lhs`, or be exactly equal to `lhs`.
/// - `dst` must not overlap `rhs`.
#[inline]
unsafe fn dispatch<const OP: u8>(dst: *mut u64, lhs: *const u64, rhs: *const u64, len: usize) {
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    {
        // SAFETY:
        // - Forwarded from `dispatch`'s safety contract.
        // - This branch is compiled only when AVX2 is enabled.
        unsafe { avx2::words::<OP>(dst, lhs, rhs, len) };
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
        unsafe { sse2::words::<OP>(dst, lhs, rhs, len) };
        return;
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        // SAFETY:
        // - Forwarded from `dispatch`'s safety contract.
        // - This branch is compiled only when NEON is enabled.
        unsafe { neon::words::<OP>(dst, lhs, rhs, len) };
        return;
    }

    #[allow(unused)]
    // SAFETY: Forwarded from `dispatch`'s safety contract.
    unsafe {
        scalar::words::<OP>(dst, lhs, rhs, len);
    }
}

#[allow(unused)]
mod scalar;

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse2;

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2;

#[allow(unused)]
#[cfg(target_arch = "aarch64")]
mod neon;
#[cfg(test)]
mod tests_for_backend_equivalence;
