//! SIMD word-level unaligned (shifted-window) comparison.
//!
//! `self` has a non-zero intra-word `shift`; `other` is word-aligned.
//! Each logical word of `self` spans two source words, reconstructed as
//! `(src[i] >> shift) | (src[i+1] << (WORD_BITS - shift))`.

use core::cmp::Ordering;

use crate::traits::WordOrd;
use crate::{SMALL_WORDS, WORD_BITS};

/// Returns `Some(Ordering)` at the first differing word, or `None` when all
/// `count` shifted windows match `other`.
#[inline]
pub(super) fn cmp_unaligned_words(
    src: &[u64],
    other: &[u64],
    count: usize,
    shift: usize,
) -> Option<Ordering> {
    debug_assert!(shift > 0 && shift < WORD_BITS);

    if count < SMALL_WORDS {
        return scalar_cmp_unaligned(src, other, count, shift);
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    {
        // SAFETY: pointer validity guaranteed by caller. Backend is always safe /
        // enabled by `#[target_feature]` at compile time.
        return unsafe { avx2::cmp_unaligned(src, other, count, shift) };
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse4.1",
        not(target_feature = "avx2")
    ))]
    {
        // SAFETY: pointer validity guaranteed by caller. Backend is always safe /
        // enabled by `#[target_feature]` at compile time.
        return unsafe { sse41::cmp_unaligned(src, other, count, shift) };
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        // SAFETY: pointer validity guaranteed by caller. Backend is always safe /
        // enabled by `#[target_feature]` at compile time.
        return unsafe { neon::cmp_unaligned(src, other, count, shift) };
    }

    #[allow(unreachable_code)]
    scalar_cmp_unaligned(src, other, count, shift)
}

#[inline]
fn scalar_cmp_unaligned(
    src: &[u64],
    other: &[u64],
    count: usize,
    shift: usize,
) -> Option<Ordering> {
    for i in 0..count {
        let w0 = src[i];
        let w1 = src[i + 1];
        let window = (w0 >> shift) | (w1 << (WORD_BITS - shift));
        if window != other[i] {
            return Some(WordOrd::bitwise_cmp(window, other[i]));
        }
    }
    None
}

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2;

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse41;

#[allow(unused)]
#[cfg(target_arch = "aarch64")]
mod neon;

#[cfg(test)]
mod tests_for_backend_equivalence;
