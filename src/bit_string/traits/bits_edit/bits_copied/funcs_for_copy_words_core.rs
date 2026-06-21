//! Word-aligned bulk copy — thin wrapper over `copy_from_slice` (memcpy).

/// Copy `count` words from `src` to `dst`.
///
/// Both slices must have at least `count` words.  This is a zero-cost
/// wrapper over [`copy_from_slice`](slice::copy_from_slice) which LLVM
/// lowers to an optimized memcpy.
#[inline]
pub(super) fn copy_words(dst: &mut [u64], src: &[u64], count: usize) {
    dst[..count].copy_from_slice(&src[..count]);
}

#[cfg(test)]
mod tests_for_backend_equivalence;
