//! SIMD shifted-window copy.
//!
//! Copies `count` shifted 64-bit windows from `src` to `dst`:
//!
//! ```text
//! dst[i] = (src[i] >> shift) | (src[i + 1] << (WORD_BITS - shift))
//! ```
//!
//! The destination is always word-aligned.  Short inputs fall back to scalar.

use crate::SMALL_WORDS;
use crate::WORD_BITS;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Copy `count` shifted 64-bit windows from `src` into `dst`.
///
/// The caller guarantees `dst` has room for `count` words, `src` has at
/// least `count + 1` words, and `shift ∈ [1, WORD_BITS)`.
#[inline]
pub(super) fn copy_words_shifted(dst: &mut [u64], src: &[u64], count: usize, shift: usize) {
    debug_assert!(shift > 0 && shift < WORD_BITS);

    if count < SMALL_WORDS {
        scalar::copy_words_shifted(dst, src, count, shift);
        return;
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    {
        // SAFETY: `dst`/`src` are valid for `count` words (caller guarantee). AVX2 availability is confirmed at compile time.
        unsafe { avx2::copy_words_shifted(dst, src, count, shift) };
        return;
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse2",
        not(target_feature = "avx2")
    ))]
    {
        // SAFETY: `dst`/`src` are valid for `count` words (caller guarantee). SSE2 availability is confirmed at compile time.
        unsafe { sse2::copy_words_shifted(dst, src, count, shift) };
        return;
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        // SAFETY: `dst`/`src` are valid for `count` words (caller guarantee). NEON availability is confirmed at compile time.
        unsafe { neon::copy_words_shifted(dst, src, count, shift) };
        return;
    }

    #[allow(unused)]
    {
        scalar::copy_words_shifted(dst, src, count, shift);
    }
}

// ---------------------------------------------------------------------------
// Backends
// ---------------------------------------------------------------------------

#[allow(unused)]
mod scalar;

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2;

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse2;

#[allow(unused)]
#[cfg(target_arch = "aarch64")]
mod neon;

#[cfg(test)]
mod tests_for_backend_equivalence;
