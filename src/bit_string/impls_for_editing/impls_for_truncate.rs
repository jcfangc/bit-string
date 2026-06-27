use crate::funcs_for_bits::*;
use crate::traits::*;

use super::*;

impl BitString {
    /// Truncates `self` to `len` bits, discarding the tail.
    ///
    /// `len` is clamped to `self.bit_len()` — a larger value is a no-op.
    #[inline]
    pub fn truncate(&mut self, len: usize) {
        let len = len.min(self.bit_len);

        if len == self.bit_len {
            return;
        }

        self.bit_len = len;

        let words = word_len(len);
        if words < self.words.len() {
            self.truncate_words(words);
        }

        self.words.mask_unused_bits(len);
    }

    /// Clears all bits, resetting `self` to an empty bit string.
    #[inline]
    pub fn clear(&mut self) {
        self.truncate_words(0);
        self.bit_len = 0;
    }
}

#[cfg(test)]
mod tests_for_truncate;
