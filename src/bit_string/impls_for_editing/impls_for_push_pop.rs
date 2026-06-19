use crate::bit_string::bits::*;

use super::*;

impl BitString {
    /// Appends a single bit to the end.
    ///
    /// Panics if the bit string length would overflow `usize`.
    #[inline]
    pub fn push(&mut self, value: bool) {
        let new_len = self
            .bit_len
            .checked_add(1)
            .expect("bit string length overflow");
        let new_words = word_len(new_len);

        // Vec::resize grows amortized O(1) — it only reallocates when
        // crossing a capacity boundary, using Vec's doubling strategy.
        if new_words > self.words.len() {
            self.words.resize(new_words, 0);
        }

        if value {
            self.words.set_bit_at(self.bit_len, true);
        }

        self.bit_len = new_len;
    }

    /// Removes and returns the last bit, or `None` if the bit string is empty.
    pub fn pop(&mut self) -> Option<bool> {
        let index = self.bit_len.checked_sub(1)?;
        let value = self.words.read_bit_at(index);

        self.words.set_bit_at(index, false);
        self.bit_len = index;

        let words = word_len(self.bit_len);
        if words < self.words.len() {
            self.truncate_words(words);
            // No mask: when pop crosses a word boundary, the new bit_len is
            // always a multiple of WORD_BITS (k×64), so mask_unused_bits is
            // a no-op (last_word_mask returns u64::MAX).
        } else {
            self.words.mask_unused_bits(self.bit_len);
        }

        Some(value)
    }
}

#[cfg(test)]
mod tests_for_push;

#[cfg(test)]
mod tests_for_pop;
