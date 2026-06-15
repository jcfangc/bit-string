use crate::bit_string::bits::Bits;

use super::*;

impl BitString {
    pub fn truncate(&mut self, len: usize) {
        assert!(
            len <= self.len,
            "cannot truncate bit string from len {} to larger len {}",
            self.len,
            len
        );

        if len == self.len {
            return;
        }

        self.len = len;

        let words = Bits::word_len(len);
        if words < self.bits.len() {
            self.bits.truncate(words);
            // Lazy shrink: only reclaim memory when capacity exceeds 2× needed.
            if self.bits.capacity() > words * 2 {
                self.bits.shrink_to(words);
            }
        }

        Bits::mask_unused(&mut self.bits, len);
    }

    pub fn clear(&mut self) {
        self.bits.clear();
        self.bits.shrink_to(0);
        self.len = 0;
    }
}

#[cfg(test)]
mod tests_for_truncate;
