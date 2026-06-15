use super::*;
use crate::bit_string::bits::Bits;

impl BitString {
    /// Constructs a bit string from packed little-endian words.
    ///
    /// The input must contain exactly enough words for `len`.
    /// Unused high bits in the last word are masked out.
    pub fn from_words(words: &[u64], len: usize) -> Option<Self> {
        let word_count = Bits::word_len(len);

        if words.len() != word_count {
            return None;
        }

        let mut bits = words.to_vec();
        Bits::mask_unused(&mut bits, len);

        Some(Self { bits, len })
    }
}

#[cfg(test)]
mod tests_for_from_words;
