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
                .copy_bits(0, rhs.bit_len)
                .paste_to(&mut self.words, old_len);
            self.bit_len = new_len;
            self.words.mask_unused_bits(self.bit_len);
            return;
        }

        // Slow path: reallocate.
        let mut bits = zero_words(new_words);

        self.words.copy_bits(0, self.bit_len).paste_to(&mut bits, 0);
        rhs.words
            .copy_bits(0, rhs.bit_len)
            .paste_to(&mut bits, old_len);

        self.words = bits;
        self.bit_len = new_len;
    }

    pub fn insert_bit_string(&mut self, index: usize, rhs: &Self) {
        let index = index.min(self.bit_len);

        if rhs.bit_len == 0 {
            return;
        }

        if index == self.bit_len {
            self.push_bit_string(rhs);
            return;
        }

        let old_len = self.bit_len;
        let shift = rhs.bit_len;
        let new_len = old_len
            .checked_add(shift)
            .expect("bit string length overflow");
        let new_words = word_len(new_len);

        // In-place fast path: grow into existing spare capacity.
        // Requires shift >= WORD_BITS so that read and write regions within
        // the same Vec never alias inside the same u64.
        if self.words.capacity() >= new_words && shift >= WORD_BITS {
            let tail_len = old_len - index;
            self.words.resize(new_words, 0);

            // Shift the tail right by `shift` bits.  Process right-to-left so
            // writes never clobber unread source data.
            let mut offset = tail_len;
            while offset > 0 {
                offset -= WORD_BITS.min(offset);
                let take = WORD_BITS.min(tail_len - offset);
                let chunk = self.words.read_word_at(index + offset);
                self.words.clear_bits_at(index + shift + offset, take);
                self.words
                    .write_word_at(index + shift + offset, chunk, take);
            }

            // The source region [index, index+shift) still holds the original
            // leading tail bits — clear it before pasting the replacement.
            self.words.clear_bits_at(index, shift);
            rhs.words
                .copy_bits(0, shift)
                .paste_to(&mut self.words, index);

            self.bit_len = new_len;
            self.words.mask_unused_bits(self.bit_len);
            return;
        }

        // Slow path: reallocate.
        let mut bits = zero_words(new_words);

        self.words.copy_bits(0, index).paste_to(&mut bits, 0);
        rhs.words.copy_bits(0, shift).paste_to(&mut bits, index);
        self.words
            .copy_bits(index, old_len - index)
            .paste_to(&mut bits, index + shift);

        self.words = bits;
        self.bit_len = new_len;
    }

    pub fn split_off(&mut self, at: usize) -> Self {
        let at = at.min(self.bit_len);

        let rhs_len = self.bit_len - at;
        let mut rhs_bits = zero_words(word_len(rhs_len));

        self.words.copy_bits(at, rhs_len).paste_to(&mut rhs_bits, 0);
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
