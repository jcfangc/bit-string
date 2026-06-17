use crate::bit_string::bits::Bits;

use super::*;

impl BitString {
    pub fn set(&mut self, index: usize, value: bool) -> Option<bool> {
        if index >= self.bit_len {
            return None;
        }

        let old = Bits::read_a_bit_at(&self.words, index);
        Bits::set_a_bit_at(&mut self.words, index, value);
        Some(old)
    }

    /// Writes `len` bits of `value` starting at `bit_start`, OR-ing them
    /// with the existing bits.  Bits beyond `self.len()` are ignored.
    ///
    /// Only the low `len` bits of `value` are used; higher bits are
    /// masked out.
    #[inline]
    pub fn set_chunk(&mut self, bit_start: usize, value: u64, len: usize) {
        let value = value & Bits::low_mask(len);
        let word = bit_start / WORD_BITS;
        let shift = bit_start % WORD_BITS;

        if let Some(w) = self.words.get_mut(word) {
            *w |= value << shift;
        }

        if shift != 0 {
            if let Some(w) = self.words.get_mut(word + 1) {
                *w |= value >> (WORD_BITS - shift);
            }
        }
    }
}

#[cfg(test)]
mod tests_for_set;
#[cfg(test)]
mod tests_for_set_chunk;
