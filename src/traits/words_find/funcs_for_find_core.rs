//! SIMD first-word pre-filter for `find`.
//!
//! Scanning order is **word-outer, shift-inner** so positions are visited
//! in increasing order and `find` returns the earliest match.

use crate::SMALL_WORDS;
use crate::WORD_BITS;
use crate::funcs_for_bits::low_mask;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[inline]
pub(super) fn find_first_word<F>(
    haystack: &[u64],
    haystack_bit_len: usize,
    needle_words: &[u64],
    needle_bit_len: usize,
    verify: &mut F,
) -> Option<usize>
where
    F: FnMut(usize) -> bool,
{
    let needle_first = needle_words[0];
    let needle_mask = low_mask(needle_bit_len.min(WORD_BITS));
    let last_start = haystack_bit_len - needle_bit_len;
    if haystack.len() < SMALL_WORDS {
        return scalar::scalar_find(haystack, needle_first, needle_mask, last_start, verify);
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    {
        // SAFETY: pointer validity guaranteed by caller. Backend is always safe /
        // enabled by `#[target_feature]` at compile time.
        return unsafe { avx2::find(haystack, needle_first, needle_mask, last_start, verify) };
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse4.1",
        not(target_feature = "avx2")
    ))]
    {
        // SAFETY: pointer validity guaranteed by caller. Backend is always safe /
        // enabled by `#[target_feature]` at compile time.
        return unsafe { sse41::find(haystack, needle_first, needle_mask, last_start, verify) };
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        // SAFETY: pointer validity guaranteed by caller. Backend is always safe /
        // enabled by `#[target_feature]` at compile time.
        return unsafe { neon::find(haystack, needle_first, needle_mask, last_start, verify) };
    }

    #[allow(unused)]
    scalar::scalar_find(haystack, needle_first, needle_mask, last_start, verify)
}

#[allow(unused)]
mod scalar;

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse41;

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2;

#[allow(unused)]
#[cfg(target_arch = "aarch64")]
mod neon;

#[cfg(test)]
mod tests_for_backend_equivalence;
