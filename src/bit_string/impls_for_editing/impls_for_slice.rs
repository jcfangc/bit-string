use int_interval::UsizeCO;

use crate::bit_string::bits::*;

use super::*;

impl BitString {
    /// Returns a new [`BitString`] containing the bits in `interval`.
    ///
    /// The interval is clamped to `[0, self.bit_len()]`.  An interval that lies
    /// entirely beyond the bit string returns an empty result.
    #[inline]
    pub fn slice(&self, interval: UsizeCO) -> Self {
        let start = interval.start().min(self.bit_len);
        let end = interval.end_excl().min(self.bit_len).max(start);
        let len = end - start;
        if len == 0 {
            return Self::new();
        }

        let mut bits = zero_words(word_len(len));
        self.words.copy_bits(start, len).paste_to(&mut bits, 0);

        Self {
            words: bits,
            bit_len: len,
        }
    }

    /// Returns a new [`BitString`] containing bits from `start` to the end.
    ///
    /// `start` is clamped to `self.bit_len()` — an out-of-bounds index returns
    /// an empty result.
    #[inline]
    pub fn slice_from(&self, start: usize) -> Self {
        let start = start.min(self.bit_len);
        let len = self.bit_len - start;
        if len == 0 {
            return Self::new();
        }

        let mut bits = zero_words(word_len(len));
        self.words.copy_bits(start, len).paste_to(&mut bits, 0);

        Self {
            words: bits,
            bit_len: len,
        }
    }

    /// Returns a new [`BitString`] containing bits from the start to `end`.
    ///
    /// `end` is clamped to `self.bit_len()` — a value beyond the bit string
    /// returns all bits.
    #[inline]
    pub fn slice_until(&self, end: usize) -> Self {
        let end = end.min(self.bit_len);
        if end == 0 {
            return Self::new();
        }

        let mut bits = zero_words(word_len(end));
        self.words.copy_bits(0, end).paste_to(&mut bits, 0);

        Self {
            words: bits,
            bit_len: end,
        }
    }
}

#[cfg(test)]
mod tests_for_slice;
