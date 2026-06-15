use crate::bit_string::bits::Bits;

use super::*;

impl BitString {
    pub fn push(&mut self, value: bool) {
        let new_len = self.len.checked_add(1).expect("bit string length overflow");
        let new_words = Bits::word_len(new_len);

        // Vec::resize grows amortized O(1) — it only reallocates when
        // crossing a capacity boundary, using Vec's doubling strategy.
        if new_words > self.bits.len() {
            self.bits.resize(new_words, 0);
        }

        if value {
            Bits::set_bit(&mut self.bits, self.len, true);
        }

        self.len = new_len;
    }

    pub fn pop(&mut self) -> Option<bool> {
        let index = self.len.checked_sub(1)?;
        let value = Bits::bit_at(&self.bits, index);

        Bits::set_bit(&mut self.bits, index, false);
        self.len = index;

        let words = Bits::word_len(self.len);
        if words < self.bits.len() {
            self.bits.truncate(words);
            // Lazy shrink: only reclaim memory when capacity exceeds 2× needed.
            if self.bits.capacity() > words * 2 {
                self.bits.shrink_to(words);
            }
        } else {
            Bits::mask_unused(&mut self.bits, self.len);
        }

        Some(value)
    }
}

#[cfg(test)]
mod tests_for_push;

#[cfg(test)]
mod tests_for_pop;
