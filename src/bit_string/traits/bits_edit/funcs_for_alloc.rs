use alloc::vec::Vec;

use crate::WORD_BITS;

/// Returns the number of `u64` words needed to store `bit_len` bits.
#[inline]
pub(crate) fn word_len(bit_len: usize) -> usize {
    bit_len / WORD_BITS + usize::from(bit_len % WORD_BITS != 0)
}

/// Allocates a zero-initialized `Vec<u64>` of `words` capacity and length.
#[inline]
pub(crate) fn zero_words(words: usize) -> Vec<u64> {
    let mut bits = Vec::with_capacity(words);
    bits.resize(words, 0);
    bits
}

/// Truncates `bits` to the first `words` words, returning a new `Vec<u64>`.
#[inline]
#[allow(dead_code)]
pub(crate) fn shrink_words(bits: &[u64], words: usize) -> Vec<u64> {
    bits[..words].to_vec()
}

#[cfg(test)]
mod tests_for_shrink_words;

#[cfg(test)]
mod tests_for_word_len;

#[cfg(test)]
mod tests_for_zero_words;
