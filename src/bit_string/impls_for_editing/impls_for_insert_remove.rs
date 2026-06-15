use crate::bit_string::bits::Bits;

use super::*;

impl BitString {
    pub fn insert(&mut self, index: usize, value: bool) {
        assert!(
            index <= self.len,
            "bit string insert index out of bounds: index={}, len={}",
            index,
            self.len
        );

        if index == self.len {
            self.push(value);
            return;
        }

        let new_len = self.len.checked_add(1).expect("bit string length overflow");
        let mut bits = Bits::zero_words(Bits::word_len(new_len));

        Bits::copy(&self.bits, 0, &mut bits, 0, index);
        Bits::set_bit(&mut bits, index, value);
        Bits::copy(&self.bits, index, &mut bits, index + 1, self.len - index);

        self.bits = bits;
        self.len = new_len;
    }

    pub fn remove(&mut self, index: usize) -> bool {
        assert!(
            index < self.len,
            "bit string remove index out of bounds: index={}, len={}",
            index,
            self.len
        );

        let value = Bits::bit_at(&self.bits, index);
        let new_len = self.len - 1;
        let mut bits = Bits::zero_words(Bits::word_len(new_len));

        Bits::copy(&self.bits, 0, &mut bits, 0, index);
        Bits::copy(
            &self.bits,
            index + 1,
            &mut bits,
            index,
            self.len - index - 1,
        );

        self.bits = bits;
        self.len = new_len;

        value
    }
}

#[cfg(test)]
mod tests_for_insert;

#[cfg(test)]
mod tests_for_remove;
