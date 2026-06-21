use crate::funcs_for_bits::*;
use crate::traits::*;

use super::*;

impl BitString {
    /// Sets the bit at `index` to `value`, returning the previous bit.
    ///
    /// Returns `None` (without modifying `self`) when `index >= self.bit_len()`.
    pub fn set(&mut self, index: usize, value: bool) -> Option<bool> {
        if index >= self.bit_len {
            return None;
        }

        let old = self.words.read_bit_at(index);
        self.words.set_bit_at(index, value);
        Some(old)
    }

    /// Writes `len` bits of `value` starting at `bit_start`, OR-ing them
    /// with the existing bits.  Bits beyond `self.len()` are ignored.
    ///
    /// Only the low `len` bits of `value` are used; higher bits are
    /// masked out.
    #[inline]
    pub fn set_chunk(&mut self, bit_start: usize, value: u64, len: usize) {
        let value = value & low_mask(len);
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
