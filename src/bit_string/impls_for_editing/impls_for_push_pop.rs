use crate::bit_string::bits::Bits;

use super::*;

impl BitString {
    pub fn push(&mut self, value: bool) {
        let new_len = self
            .bit_len
            .checked_add(1)
            .expect("bit string length overflow");
        let new_words = Bits::word_len(new_len);

        // Vec::resize grows amortized O(1) — it only reallocates when
        // crossing a capacity boundary, using Vec's doubling strategy.
        if new_words > self.words.len() {
            self.words.resize(new_words, 0);
        }

        if value {
            Bits::set_a_bit_at(&mut self.words, self.bit_len, true);
        }

        self.bit_len = new_len;
    }

    pub fn pop(&mut self) -> Option<bool> {
        let index = self.bit_len.checked_sub(1)?;
        let value = Bits::read_a_bit_at(&self.words, index);

        Bits::set_a_bit_at(&mut self.words, index, false);
        self.bit_len = index;

        let words = Bits::word_len(self.bit_len);
        if words < self.words.len() {
            self.words.truncate(words);
            // Lazy shrink: only reclaim memory when capacity exceeds 2× needed.
            if self.words.capacity() > words * 2 {
                self.words.shrink_to(words);
            }
        } else {
            Bits::mask_unused(&mut self.words, self.bit_len);
        }

        Some(value)
    }
}

#[cfg(test)]
mod tests_for_push;

#[cfg(test)]
mod tests_for_pop;
