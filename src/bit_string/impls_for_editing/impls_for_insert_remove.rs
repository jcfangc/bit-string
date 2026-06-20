use crate::bit_string::traits::*;

use super::*;

impl BitString {
    /// Inserts a single bit at `index`, shifting the tail right.
    ///
    /// `index` is clamped to `[0, self.bit_len()]` — an out-of-bounds index
    /// pushes at the end.
    pub fn insert(&mut self, index: usize, value: bool) {
        let index = index.min(self.bit_len);

        if index == self.bit_len {
            self.push(value);
            return;
        }

        let new_len = self
            .bit_len
            .checked_add(1)
            .expect("bit string length overflow");
        let new_words = word_len(new_len);

        // In-place fast path: word count unchanged (~98% of operations).
        if new_words == self.words.len() {
            // Ensure the last word is present (Vec::len == new_words already).
            self.words.resize(new_words, 0);
            self.words.shift_right_in_place(index, self.bit_len - index);
            self.words.set_bit_at(index, value);
            self.bit_len = new_len;
            return;
        }

        // Word count changed — allocate a fresh buffer.
        let mut bits = zero_words(new_words);

        self.words.copy_bits(0, index).paste_to(&mut bits, 0);
        bits.set_bit_at(index, value);
        self.words
            .copy_bits(index, self.bit_len - index)
            .paste_to(&mut bits, index + 1);

        self.words = bits;
        self.bit_len = new_len;
    }

    /// Removes the bit at `index`, shifting the tail left.
    ///
    /// Returns `false` (without modifying `self`) when `index >= self.bit_len()`.
    pub fn remove(&mut self, index: usize) -> bool {
        if index >= self.bit_len {
            return false;
        }

        let value = self.words.read_bit_at(index);
        let new_len = self.bit_len - 1;
        let new_words = word_len(new_len);

        // In-place fast path: word count unchanged.
        if new_words == self.words.len() {
            self.words
                .shift_left_in_place(index + 1, self.bit_len - index - 1);
            self.bit_len = new_len;
            self.words.mask_unused_bits(self.bit_len);
            return value;
        }

        // Word count changed — allocate a fresh buffer.
        let mut bits = zero_words(new_words);

        self.words.copy_bits(0, index).paste_to(&mut bits, 0);
        self.words
            .copy_bits(index + 1, self.bit_len - index - 1)
            .paste_to(&mut bits, index);

        self.words = bits;
        self.bit_len = new_len;

        value
    }
}

#[cfg(test)]
mod tests_for_insert;

#[cfg(test)]
mod tests_for_remove;
