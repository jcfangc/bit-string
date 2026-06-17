use crate::bit_string::bits::*;

use super::*;

impl BitString {
    pub fn truncate(&mut self, len: usize) {
        assert!(
            len <= self.bit_len,
            "cannot truncate bit string from len {} to larger len {}",
            self.bit_len,
            len
        );

        if len == self.bit_len {
            return;
        }

        self.bit_len = len;

        let words = word_len(len);
        if words < self.words.len() {
            self.words.truncate(words);
            // Lazy shrink: only reclaim memory when capacity exceeds 2× needed.
            if self.words.capacity() > words * 2 {
                self.words.shrink_to(words);
            }
        }

        self.words.mask_unused_bits(len);
    }

    pub fn clear(&mut self) {
        self.words.clear();
        self.words.shrink_to(0);
        self.bit_len = 0;
    }
}

#[cfg(test)]
mod tests_for_truncate;
