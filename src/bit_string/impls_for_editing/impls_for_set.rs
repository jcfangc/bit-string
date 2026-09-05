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

    /// Overwrites `len` bits starting at `bit_start` with the low bits of
    /// `value`. Bits beyond `self.len()` are ignored.
    ///
    /// Only the low `len` bits of `value` are used; higher bits are
    /// masked out.
    #[inline]
    pub fn set_chunk(&mut self, bit_start: usize, value: u64, len: usize) {
        let len = len
            .min(WORD_BITS)
            .min(self.bit_len.saturating_sub(bit_start));
        if len == 0 {
            return;
        }

        self.words.clear_bits_at(bit_start, len);
        self.words.write_word_at::<false>(bit_start, value, len);
        self.words.mask_unused_bits(self.bit_len);
    }
}

#[cfg(test)]
mod tests_for_set;
#[cfg(test)]
mod tests_for_set_chunk;
