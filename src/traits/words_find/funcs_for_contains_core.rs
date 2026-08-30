//! SIMD first-word pre-filter for `contains`, `find`, and `rfind`.
//!
//! Returns `Some(pos)` for the first candidate whose 64-bit window
//! matches the needle's first word AND `verify(pos)` succeeds.
//!
//! Uses **shift-outer, word-inner** ordering, processing LANES haystack
//! words in parallel per shift.  This ordering does **not** guarantee
//! the returned position is the earliest match — `find` must use a
//! binary-search driver or a word-outer scan for correct ordering.

use crate::SMALL_WORDS;
use crate::WORD_BITS;
use crate::funcs_for_bits::low_mask;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Returns `Some(pos)` if any 64-bit window in `haystack[0..word_limit]`
/// matches the first word of `needle_words` AND `verify(pos)` succeeds.
///
/// Scans positions `pos ∈ [0, haystack_bit_len - needle_bit_len]` using
/// **shift-outer, word-inner** ordering — does **not** guarantee the
/// returned position is the earliest match.
#[inline]
pub(super) fn find_any_candidate<F>(
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
    let max_word = last_start / WORD_BITS;
    let word_limit = (max_word + 1).min(haystack.len());

    if haystack.len() < SMALL_WORDS {
        return scalar(
            haystack,
            needle_first,
            needle_mask,
            last_start,
            word_limit,
            verify,
        );
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    {
        // SAFETY: pointer validity guaranteed by caller. Backend is always safe /
        // enabled by `#[target_feature]` at compile time.
        return unsafe {
            avx2::find_any(
                haystack,
                needle_first,
                needle_mask,
                last_start,
                word_limit,
                verify,
            )
        };
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse4.1",
        not(target_feature = "avx2")
    ))]
    {
        // SAFETY: pointer validity guaranteed by caller. Backend is always safe /
        // enabled by `#[target_feature]` at compile time.
        return unsafe {
            sse41::find_any(
                haystack,
                needle_first,
                needle_mask,
                last_start,
                word_limit,
                verify,
            )
        };
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        // SAFETY: pointer validity guaranteed by caller. Backend is always safe /
        // enabled by `#[target_feature]` at compile time.
        return unsafe {
            neon::find_any(
                haystack,
                needle_first,
                needle_mask,
                last_start,
                word_limit,
                verify,
            )
        };
    }

    #[allow(unused)]
    scalar(
        haystack,
        needle_first,
        needle_mask,
        last_start,
        word_limit,
        verify,
    )
}

// ---------------------------------------------------------------------------
// Scalar fallback
// ---------------------------------------------------------------------------

/// Word-by-word scan: for each shift, check every word pair in
/// `[0, word_limit)` for a matching window.
fn scalar<F>(
    haystack: &[u64],
    needle_first: u64,
    needle_mask: u64,
    last_start: usize,
    word_limit: usize,
    verify: &mut F,
) -> Option<usize>
where
    F: FnMut(usize) -> bool,
{
    for shift in 0..WORD_BITS {
        for i in 0..word_limit {
            let pos = i * WORD_BITS + shift;
            if pos > last_start {
                break;
            }
            let window = if shift == 0 {
                haystack[i]
            } else {
                let w0 = haystack[i];
                let w1 = haystack.get(i + 1).copied().unwrap_or(0);
                (w0 >> shift) | (w1 << (WORD_BITS - shift))
            };
            if (window & needle_mask) == needle_first && verify(pos) {
                return Some(pos);
            }
        }
    }
    None
}

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
