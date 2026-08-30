//! SIMD word-level unaligned (shifted-window) equality.
//!
//! Computes shifted 64-bit windows and compares each against the
//! corresponding word in `other`.  The caller must ensure `shift != 0`;
//! the `shift == 0` fast path is handled by the trait impl.

use crate::SMALL_WORDS;
use crate::WORD_BITS;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[inline]
pub(super) fn eq_words_unaligned(src: &[u64], other: &[u64], count: usize, shift: usize) -> bool {
    debug_assert!(shift > 0 && shift < WORD_BITS);

    if count < SMALL_WORDS {
        for i in 0..count {
            let w0 = src[i];
            let w1 = src[i + 1];
            if ((w0 >> shift) | (w1 << (WORD_BITS - shift))) != other[i] {
                return false;
            }
        }
        return true;
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    {
        // SAFETY: `src`/`other` are valid for `count+1` words (caller ensures extra word for shifting). Backend feature is guaranteed by compile-time `#[cfg]` gate.
        return unsafe { avx2::eq_words_unaligned(src, other, count, shift) };
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse4.1",
        not(target_feature = "avx2")
    ))]
    {
        // SAFETY: `src`/`other` are valid for `count+1` words (caller ensures extra word for shifting). Backend feature is guaranteed by compile-time `#[cfg]` gate.
        return unsafe { sse41::eq_words_unaligned(src, other, count, shift) };
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        // SAFETY: `src`/`other` are valid for `count+1` words (caller ensures extra word for shifting). Backend feature is enabled by the `#[cfg]` gate on this block.
        return unsafe { neon::eq_words_unaligned(src, other, count, shift) };
    }

    #[allow(unused)]
    {
        for i in 0..count {
            let w0 = src[i];
            let w1 = src[i + 1];
            if ((w0 >> shift) | (w1 << (WORD_BITS - shift))) != other[i] {
                return false;
            }
        }
        true
    }
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
