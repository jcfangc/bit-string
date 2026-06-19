use crate::bit_string::bits::*;

use super::*;

impl BitString {
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

    pub fn clear(&mut self) {
        self.truncate_words(0);
        self.bit_len = 0;
    }
}

#[cfg(test)]
mod tests_for_truncate;
