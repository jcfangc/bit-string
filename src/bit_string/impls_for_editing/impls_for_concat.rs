use crate::bit_string::bits::*;

use super::*;

impl BitString {
    pub fn push_bit_string(&mut self, rhs: &Self) {
        if rhs.bit_len == 0 {
            return;
        }

        if self.bit_len == 0 {
            self.words = rhs.words.clone();
            self.bit_len = rhs.bit_len;
            return;
        }

        let old_len = self.bit_len;
        let new_len = old_len
            .checked_add(rhs.bit_len)
            .expect("bit string length overflow");
        let new_words = word_len(new_len);

        // In-place fast path: grow into existing spare capacity.
        if self.words.capacity() >= new_words {
            self.words.resize(new_words, 0);
            rhs.words
                .copy_bits_to(0, &mut self.words, old_len, rhs.bit_len);
            self.bit_len = new_len;
            self.words.mask_unused_bits(self.bit_len);
            return;
        }

        // Slow path: reallocate.
        let mut bits = zero_words(new_words);

        self.words.copy_bits_to(0, &mut bits, 0, self.bit_len);
        rhs.words.copy_bits_to(0, &mut bits, old_len, rhs.bit_len);

        self.words = bits;
        self.bit_len = new_len;
    }

    pub fn insert_bit_string(&mut self, index: usize, rhs: &Self) {
        assert!(
            index <= self.bit_len,
            "bit string insert index out of bounds: index={}, len={}",
            index,
            self.bit_len
        );

        if rhs.bit_len == 0 {
            return;
        }

        if index == self.bit_len {
            self.push_bit_string(rhs);
            return;
        }

        let new_len = self
            .bit_len
            .checked_add(rhs.bit_len)
            .expect("bit string length overflow");

        let mut bits = zero_words(word_len(new_len));

        self.words.copy_bits_to(0, &mut bits, 0, index);
        rhs.words.copy_bits_to(0, &mut bits, index, rhs.bit_len);
        self.words
            .copy_bits_to(index, &mut bits, index + rhs.bit_len, self.bit_len - index);

        self.words = bits;
        self.bit_len = new_len;
    }

    pub fn split_off(&mut self, at: usize) -> Self {
        assert!(
            at <= self.bit_len,
            "bit string split index out of bounds: index={}, len={}",
            at,
            self.bit_len
        );

        let rhs_len = self.bit_len - at;
        let mut rhs_bits = zero_words(word_len(rhs_len));

        self.words.copy_bits_to(at, &mut rhs_bits, 0, rhs_len);
        self.truncate(at);

        Self {
            words: rhs_bits,
            bit_len: rhs_len,
        }
    }
}

#[cfg(test)]
mod tests_for_push_bit_string;

#[cfg(test)]
mod tests_for_insert_bit_string;

#[cfg(test)]
mod tests_for_split_off;
