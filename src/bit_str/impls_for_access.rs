use crate::low_mask;
use crate::traits::*;

use super::*;

impl<'bs> BitStr<'bs> {
    /// Returns the bit at `index`, or `None` when `index >= self.bit_len()`.
    #[inline]
    pub fn get(&self, index: usize) -> Option<bool> {
        if index >= self.bit_len {
            return None;
        }
        Some(self.source.words().read_bit_at(self.start + index))
    }

    /// Returns the first bit, or `None` when the view is empty.
    #[inline]
    pub fn first(&self) -> Option<bool> {
        self.get(0)
    }

    /// Returns the last bit, or `None` when the view is empty.
    #[inline]
    pub fn last(&self) -> Option<bool> {
        self.bit_len.checked_sub(1).and_then(|i| self.get(i))
    }

    /// Reads up to 64 bits starting at `bit_start`, returning them in the
    /// low bits of a `u64`.
    ///
    /// Bits beyond `self.bit_len()` are treated as zero.
    #[inline]
    pub fn get_chunk(&self, bit_start: usize) -> u64 {
        let valid_bits = self.bit_len.saturating_sub(bit_start);
        if valid_bits == 0 {
            return 0;
        }
        let raw = self.source.words().read_word_at(self.start + bit_start);
        raw & low_mask(valid_bits)
    }
}

#[cfg(test)]
mod tests_for_get;
#[cfg(test)]
mod tests_for_get_chunk;
