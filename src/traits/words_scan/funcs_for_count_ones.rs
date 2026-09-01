use crate::SMALL_WORDS;
use crate::WORD_BITS;
use crate::low_mask;

#[inline]
pub(super) fn count_ones(bits: &[u64], bit_len: usize) -> usize {
    let full_words = bit_len / WORD_BITS;
    let rem = bit_len % WORD_BITS;

    // Fast path: for inputs too short to amortize SIMD setup, loop over
    // the words directly with scalar popcnt, skipping dispatch entirely.
    if full_words < SMALL_WORDS {
        let mut count = 0usize;
        for i in 0..full_words {
            count += bits[i].count_ones() as usize;
        }
        if rem != 0 {
            count += (bits[full_words] & low_mask(rem)).count_ones() as usize;
        }
        return count;
    }

    let mut count = count_full_words(&bits[..full_words]);

    if rem != 0 {
        count += (bits[full_words] & low_mask(rem)).count_ones() as usize;
    }

    count
}

#[inline]
fn count_full_words(words: &[u64]) -> usize {
    // SAFETY:
    // - `words.as_ptr()` is valid for reads of `words.len()` u64 values.
    // - `dispatch` only reads from `words[0..len]`.
    unsafe { dispatch(words.as_ptr(), words.len()) }
}

/// Counts set bits in `src[0..len]`.
///
/// # Safety
///
/// - `src` must be valid for reads of `len` initialized `u64` values.
#[inline]
unsafe fn dispatch(src: *const u64, len: usize) -> usize {
    // Small inputs: skip SIMD setup overhead, go straight to scalar popcount.
    // The threshold is selected by the active target-feature configuration.

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    {
        if len >= 4 {
            // SAFETY: `src` is valid for `len` words. Backend selected via `#[cfg]` feature gate.
            return unsafe { avx2::count_words(src, len) };
        }
    }

    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "ssse3",
        not(target_feature = "avx2")
    ))]
    {
        if len >= 2 {
            // SAFETY: `src` is valid for `len` words. Backend selected via `#[cfg]` feature gate.
            return unsafe { ssse3::count_words(src, len) };
        }
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        if len >= 2 {
            // SAFETY: `src` is valid for `len` words. Backend selected via `#[cfg]` feature gate.
            return unsafe { neon::count_words(src, len) };
        }
    }

    #[allow(unused)]
    // SAFETY: pointer validity guaranteed by caller. Scalar backend is always safe.
    unsafe {
        scalar::count_words(src, len)
    }
}

#[allow(unused)]
mod scalar;

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2;

#[allow(unused)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod ssse3;

#[allow(unused)]
#[cfg(target_arch = "aarch64")]
mod neon;

#[cfg(test)]
mod tests_for_backend_equivalence;
