use crate::bit_string::bits::Bits;

use super::*;

impl BitString {
    pub fn insert(&mut self, index: usize, value: bool) {
        assert!(
            index <= self.bit_len,
            "bit string insert index out of bounds: index={}, len={}",
            index,
            self.bit_len
        );

        if index == self.bit_len {
            self.push(value);
            return;
        }

        let new_len = self
            .bit_len
            .checked_add(1)
            .expect("bit string length overflow");
        let new_words = Bits::word_len(new_len);

        // In-place fast path: word count unchanged (~98% of operations).
        if new_words == self.words.len() {
            // Ensure the last word is present (Vec::len == new_words already).
            self.words.resize(new_words, 0);
            Bits::shift_right_in_place(&mut self.words, index, self.bit_len - index);
            Bits::set_a_bit_at(&mut self.words, index, value);
            self.bit_len = new_len;
            return;
        }

        // Word count changed — allocate a fresh buffer.
        let mut bits = Bits::zero_words(new_words);

        Bits::copy(&self.words, 0, &mut bits, 0, index);
        Bits::set_a_bit_at(&mut bits, index, value);
        Bits::copy(
            &self.words,
            index,
            &mut bits,
            index + 1,
            self.bit_len - index,
        );

        self.words = bits;
        self.bit_len = new_len;
    }

    pub fn remove(&mut self, index: usize) -> bool {
        assert!(
            index < self.bit_len,
            "bit string remove index out of bounds: index={}, len={}",
            index,
            self.bit_len
        );

        let value = Bits::read_a_bit_at(&self.words, index);
        let new_len = self.bit_len - 1;
        let new_words = Bits::word_len(new_len);

        // In-place fast path: word count unchanged.
        if new_words == self.words.len() {
            Bits::shift_left_in_place(&mut self.words, index + 1, self.bit_len - index - 1);
            self.bit_len = new_len;
            Bits::mask_unused(&mut self.words, self.bit_len);
            return value;
        }

        // Word count changed — allocate a fresh buffer.
        let mut bits = Bits::zero_words(new_words);

        Bits::copy(&self.words, 0, &mut bits, 0, index);
        Bits::copy(
            &self.words,
            index + 1,
            &mut bits,
            index,
            self.bit_len - index - 1,
        );

        self.words = bits;
        self.bit_len = new_len;

        value
    }
}

#[cfg(test)]
mod tests_for_insert;

#[cfg(test)]
mod tests_for_remove;
