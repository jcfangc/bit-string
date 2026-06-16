use crate::bit_string::bits::Bits;

use super::*;

impl BitString {
    pub fn push_bit_string(&mut self, rhs: &Self) {
        if rhs.len == 0 {
            return;
        }

        if self.len == 0 {
            self.bits = rhs.bits.clone();
            self.len = rhs.len;
            return;
        }

        let old_len = self.len;
        let new_len = old_len
            .checked_add(rhs.len)
            .expect("bit string length overflow");
        let new_words = Bits::word_len(new_len);

        // In-place fast path: grow into existing spare capacity.
        if self.bits.capacity() >= new_words {
            self.bits.resize(new_words, 0);
            Bits::copy(&rhs.bits, 0, &mut self.bits, old_len, rhs.len);
            self.len = new_len;
            Bits::mask_unused(&mut self.bits, self.len);
            return;
        }

        // Slow path: reallocate.
        let mut bits = Bits::zero_words(new_words);

        Bits::copy(&self.bits, 0, &mut bits, 0, self.len);
        Bits::copy(&rhs.bits, 0, &mut bits, old_len, rhs.len);

        self.bits = bits;
        self.len = new_len;
    }

    pub fn insert_bit_string(&mut self, index: usize, rhs: &Self) {
        assert!(
            index <= self.len,
            "bit string insert index out of bounds: index={}, len={}",
            index,
            self.len
        );

        if rhs.len == 0 {
            return;
        }

        if index == self.len {
            self.push_bit_string(rhs);
            return;
        }

        let new_len = self
            .len
            .checked_add(rhs.len)
            .expect("bit string length overflow");

        let mut bits = Bits::zero_words(Bits::word_len(new_len));

        Bits::copy(&self.bits, 0, &mut bits, 0, index);
        Bits::copy(&rhs.bits, 0, &mut bits, index, rhs.len);
        Bits::copy(
            &self.bits,
            index,
            &mut bits,
            index + rhs.len,
            self.len - index,
        );

        self.bits = bits;
        self.len = new_len;
    }

    pub fn split_off(&mut self, at: usize) -> Self {
        assert!(
            at <= self.len,
            "bit string split index out of bounds: index={}, len={}",
            at,
            self.len
        );

        let rhs_len = self.len - at;
        let mut rhs_bits = Bits::zero_words(Bits::word_len(rhs_len));

        Bits::copy(&self.bits, at, &mut rhs_bits, 0, rhs_len);
        self.truncate(at);

        Self {
            bits: rhs_bits,
            len: rhs_len,
        }
    }
}

#[cfg(test)]
mod tests_for_push_bit_string;

#[cfg(test)]
mod tests_for_insert_bit_string;

#[cfg(test)]
mod tests_for_split_off;
