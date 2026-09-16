use crate::funcs_for_bits::*;
use crate::traits::*;
use crate::{WORD_BITS, low_mask};

use super::*;

impl BitString {
    /// Reserves backing storage for an additional number of bits.
    #[inline]
    pub(crate) fn reserve_bits(&mut self, additional_bits: usize) {
        if let Some(total_bits) = self.bit_len.checked_add(additional_bits) {
            let additional_words = word_len(total_bits).saturating_sub(self.words.len());
            self.words.reserve(additional_words);
        }
    }

    /// Appends the low `bits` bits of `value` in one or two word writes.
    #[inline]
    pub(crate) fn push_bits(&mut self, value: u64, bits: usize) {
        assert!(
            bits <= WORD_BITS,
            "cannot append more than one word of bits"
        );
        if bits == 0 {
            return;
        }

        let new_bit_len = self
            .bit_len
            .checked_add(bits)
            .expect("bit string length overflow");
        let word_index = self.bit_len / WORD_BITS;
        let bit_offset = self.bit_len % WORD_BITS;

        if word_index == self.words.len() {
            self.words.push(0);
        }

        let value = value & low_mask(bits);
        self.words[word_index] |= value << bit_offset;

        let next_offset = bit_offset + bits;
        if next_offset > WORD_BITS {
            if word_index + 1 == self.words.len() {
                self.words.push(0);
            }
            self.words[word_index + 1] |= value >> (WORD_BITS - bit_offset);
        }

        self.bit_len = new_bit_len;
    }

    /// Appends a single bit to the end.
    ///
    /// Panics if the bit string length would overflow `usize`.
    #[inline]
    pub fn push(&mut self, value: bool) {
        let new_len = self
            .bit_len
            .checked_add(1)
            .expect("bit string length overflow");
        let new_words = word_len(new_len);

        // Vec::resize grows amortized O(1) — it only reallocates when
        // crossing a capacity boundary, using Vec's doubling strategy.
        if new_words > self.words.len() {
            self.words.resize(new_words, 0);
        }

        if value {
            self.words.set_bit_at(self.bit_len, true);
        }

        self.bit_len = new_len;
    }

    /// Removes and returns the last bit, or `None` if the bit string is empty.
    pub fn pop(&mut self) -> Option<bool> {
        let index = self.bit_len.checked_sub(1)?;
        let value = self.words.read_bit_at(index);

        self.words.set_bit_at(index, false);
        self.bit_len = index;

        let words = word_len(self.bit_len);
        if words < self.words.len() {
            self.truncate_words(words);
            // No mask: when pop crosses a word boundary, the new bit_len is
            // always a multiple of WORD_BITS (k×64), so mask_unused_bits is
            // a no-op (last_word_mask returns u64::MAX).
        } else {
            self.words.mask_unused_bits(self.bit_len);
        }

        Some(value)
    }
}

#[cfg(test)]
mod tests_for_push;

#[cfg(test)]
mod tests_for_pop;
