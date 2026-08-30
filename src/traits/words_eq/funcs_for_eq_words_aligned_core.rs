//! SIMD word-level equality — compares LANES u64 words at once
//! via `cmpeq` + `movemask`.

use crate::SMALL_WORDS;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Returns `true` if the first `count` words of `src` match `other`.
///
/// Dispatches to the best available SIMD backend at compile time.
/// Short inputs fall back to scalar.
#[inline]
pub(super) fn eq_words_aligned(src: &[u64], other: &[u64], count: usize) -> bool {
    if count < SMALL_WORDS {
        for i in 0..count {
            if src[i] != other[i] {
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
        // SAFETY: `src`/`other` are valid for `count` words. Backend feature is guaranteed by compile-time `#[cfg]` gate.
        return unsafe { avx2::eq_words(src, other, count) };
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse4.1",
        not(target_feature = "avx2")
    ))]
    {
        // SAFETY: `src`/`other` are valid for `count` words. Backend feature is guaranteed by compile-time `#[cfg]` gate.
        return unsafe { sse41::eq_words(src, other, count) };
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        // SAFETY: `src`/`other` are valid for `count` words. Backend feature is enabled by the `#[cfg]` gate on this block.
        return unsafe { neon::eq_words(src, other, count) };
    }

    #[allow(unused)]
    {
        for i in 0..count {
            if src[i] != other[i] {
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
