use core::cmp::Ordering;

use crate::SMALL_WORDS;

/// Returns `Some(Ordering)` if the first `count` aligned words of `src`
/// and `other` differ, otherwise `None` (all equal).
///
/// Dispatches to the best available SIMD backend at compile time.
/// Short inputs fall back to scalar.
#[inline]
pub(super) fn cmp_aligned_words(src: &[u64], other: &[u64], count: usize) -> Option<Ordering> {
    if count < SMALL_WORDS {
        return scalar::cmp_aligned(src, other, count);
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    {
        // SAFETY: pointer validity guaranteed by caller. Backend is always safe /
        // enabled by `#[target_feature]` at compile time.
        return unsafe { avx2::cmp_aligned(src, other, count) };
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse4.1",
        not(target_feature = "avx2")
    ))]
    {
        // SAFETY: pointer validity guaranteed by caller. Backend is always safe /
        // enabled by `#[target_feature]` at compile time.
        return unsafe { sse41::cmp_aligned(src, other, count) };
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        // SAFETY: pointer validity guaranteed by caller. Backend is always safe /
        // enabled by `#[target_feature]` at compile time.
        return unsafe { neon::cmp_aligned(src, other, count) };
    }

    #[allow(unreachable_code)]
    scalar::cmp_aligned(src, other, count)
}

#[allow(unused)]
mod scalar;

// ---------------------------------------------------------------------------
// AVX2 — 4 × u64 per iteration
// ---------------------------------------------------------------------------

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2;

// ---------------------------------------------------------------------------
// SSE2 — 2 × u64 per iteration
// ---------------------------------------------------------------------------

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse41;

// ---------------------------------------------------------------------------
// NEON — 2 × u64 per iteration
// ---------------------------------------------------------------------------

#[allow(unused)]
#[cfg(target_arch = "aarch64")]
mod neon;

#[cfg(test)]
mod tests_for_backend_equivalence;
