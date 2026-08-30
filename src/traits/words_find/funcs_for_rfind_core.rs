//! SIMD first-word pre-filter for `rfind`.
//!
//! Scanning order is **word-outer reverse, shift-inner reverse** so
//! positions are visited in decreasing order and `rfind` returns the
//! rightmost match.

use crate::SMALL_WORDS;
use crate::WORD_BITS;
use crate::funcs_for_bits::low_mask;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[inline]
pub(super) fn find_last_word<F>(
    haystack: &[u64],
    haystack_bit_len: usize,
    needle_words: &[u64],
    needle_bit_len: usize,
    verify: &mut F,
) -> Option<usize>
where
    F: FnMut(usize) -> bool,
{
    let needle_key = needle_words[0];
    let needle_mask = low_mask(needle_bit_len.min(WORD_BITS));
    let last_start = haystack_bit_len - needle_bit_len;

    if haystack.len() < SMALL_WORDS {
        return scalar_rfind(haystack, needle_key, needle_mask, last_start, verify);
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    {
        // SAFETY: pointer validity guaranteed by caller. Backend is always safe /
        // enabled by `#[target_feature]` at compile time.
        return unsafe { avx2::rfind(haystack, needle_key, needle_mask, last_start, verify) };
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse4.1",
        not(target_feature = "avx2")
    ))]
    {
        // SAFETY: pointer validity guaranteed by caller. Backend is always safe /
        // enabled by `#[target_feature]` at compile time.
        return unsafe { sse41::rfind(haystack, needle_key, needle_mask, last_start, verify) };
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        // SAFETY: pointer validity guaranteed by caller. Backend is always safe /
        // enabled by `#[target_feature]` at compile time.
        return unsafe { neon::rfind(haystack, needle_key, needle_mask, last_start, verify) };
    }

    #[allow(unused)]
    scalar_rfind(haystack, needle_key, needle_mask, last_start, verify)
}

// ---------------------------------------------------------------------------
// Scalar
// ---------------------------------------------------------------------------

fn scalar_rfind<F>(
    haystack: &[u64],
    needle_key: u64,
    needle_mask: u64,
    last_start: usize,
    verify: &mut F,
) -> Option<usize>
where
    F: FnMut(usize) -> bool,
{
    let start_word = (last_start / WORD_BITS).min(haystack.len().saturating_sub(1));
    for i in (0..=start_word).rev() {
        let base = i * WORD_BITS;
        let w0 = haystack[i];
        let w1 = haystack.get(i + 1).copied().unwrap_or(0);
        // Note: the SIMD backends compute max_shift differently —
        // `WORD_BITS.min(last_start - base + 1)` — to process
        // shifts in SIMD-sized chunks (2 or 4), relying on
        // `pos <= last_start` to skip out-of-range positions.
        let max_shift = (last_start - base).min(WORD_BITS - 1);
        for shift in (0..=max_shift).rev() {
            let pos = base + shift;
            let window = if shift == 0 {
                w0
            } else {
                (w0 >> shift) | (w1 << (WORD_BITS - shift))
            };
            if (window & needle_mask) == needle_key && verify(pos) {
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
