use alloc::vec::Vec;

use crate::WORD_BITS;

/// Returns `u64::MAX` when `bits >= WORD_BITS`, otherwise the low `bits` ones.
#[inline]
pub(crate) fn low_mask(bits: usize) -> u64 {
    if bits >= WORD_BITS {
        u64::MAX
    } else {
        (1u64 << bits) - 1
    }
}

/// Returns the mask for the last word of a bit string of total length `len`.
///
/// The number of valid bits in the last word is `len % WORD_BITS`. When that
/// remainder is zero the last word is full and `u64::MAX` is returned;
/// otherwise only the low `len % WORD_BITS` bits are set.
#[inline]
pub(crate) fn last_word_mask(len: usize) -> u64 {
    let rem = len % WORD_BITS;
    if rem == 0 {
        u64::MAX
    } else {
        (1u64 << rem) - 1
    }
}

/// Returns the number of `u64` words needed to store `bit_len` bits.
#[inline]
pub(crate) fn word_len(bit_len: usize) -> usize {
    bit_len / WORD_BITS + usize::from(bit_len % WORD_BITS != 0)
}

/// Below this many full words, scalar loops beat SIMD dispatch overhead.
/// Must match each backend's `LANES`.
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
pub(crate) const SMALL_WORDS: usize = 4;
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "ssse3",
    not(target_feature = "avx2")
))]
pub(crate) const SMALL_WORDS: usize = 2;
#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
pub(crate) const SMALL_WORDS: usize = 2;
#[cfg(not(any(
    all(
        any(target_arch = "x86", target_arch = "x86_64"),
        any(target_feature = "avx2", target_feature = "ssse3")
    ),
    all(target_arch = "aarch64", target_feature = "neon"),
)))]
pub(crate) const SMALL_WORDS: usize = 0;

/// Allocates a zero-initialized `Vec<u64>` of `words` capacity and length.
#[inline]
pub(crate) fn zero_words(words: usize) -> Vec<u64> {
    let mut bits = Vec::with_capacity(words);
    bits.resize(words, 0);
    bits
}

#[cfg(test)]
mod tests_for_low_mask;

#[cfg(test)]
mod tests_for_last_word_mask;

#[cfg(test)]
mod tests_for_word_len;

#[cfg(test)]
mod tests_for_zero_words;
