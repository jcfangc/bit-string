use alloc::vec::Vec;

use super::*;
use crate::funcs_for_bits::*;
use crate::traits::*;

impl BitString {
    /// Constructs a bit string from packed little-endian words.
    ///
    /// The input must contain exactly enough words for `len`.
    /// Unused high bits in the last word are masked out.
    pub fn from_words(words: &[u64], len: usize) -> Option<Self> {
        let word_count = word_len(len);

        if words.len() != word_count {
            return None;
        }

        let mut bits = words.to_vec();
        bits.mask_unused_bits(len);

        Some(Self {
            words: bits,
            bit_len: len,
        })
    }

    /// Constructs a bit string by taking ownership of packed little-endian words.
    ///
    /// The input must contain exactly enough words for `len`. Unused high bits
    /// in the last word are masked out before the vector is adopted.
    #[inline]
    pub(crate) fn from_owned_words(mut words: Vec<u64>, len: usize) -> Option<Self> {
        if words.len() != word_len(len) {
            return None;
        }

        words.mask_unused_bits(len);

        Some(Self {
            words,
            bit_len: len,
        })
    }
}

#[cfg(test)]
mod tests_for_from_words;
