use int_interval::UsizeCO;

use crate::bit_string::bits::*;

use super::*;

impl BitString {
    /// Returns a new [`BitString`] containing the bits in `interval`.
    ///
    /// # Panics
    ///
    /// Panics if `interval` is not fully within `[0, self.bit_len()]`.
    /// For a panics-free variant, use [`drain_interval`](Self::drain_interval)
    /// which clamps the interval.
    #[inline]
    pub fn slice(&self, interval: UsizeCO) -> Self {
        assert_interval_in_bounds(interval, self.bit_len);

        let start = interval.start();
        let len = interval.len();

        let mut bits = zero_words(word_len(len));
        self.words.copy_bits(start, len).paste_to(&mut bits, 0);

        Self {
            words: bits,
            bit_len: len,
        }
    }

    /// Returns a new [`BitString`] containing bits from `start` to the end.
    ///
    /// # Panics
    ///
    /// Panics if `start > self.bit_len()`.
    #[inline]
    pub fn slice_from(&self, start: usize) -> Self {
        assert!(
            start <= self.bit_len,
            "bit string slice start out of bounds: start={}, len={}",
            start,
            self.bit_len
        );

        let len = self.bit_len - start;

        if len == 0 {
            return Self::new();
        }

        let interval = UsizeCO::checked_from_start_len(start, len).unwrap();
        self.slice(interval)
    }

    /// Returns a new [`BitString`] containing bits from the start to `end`.
    ///
    /// # Panics
    ///
    /// Panics if `end > self.bit_len()`.
    #[inline]
    pub fn slice_until(&self, end: usize) -> Self {
        assert!(
            end <= self.bit_len,
            "bit string slice end out of bounds: end={}, len={}",
            end,
            self.bit_len
        );

        if end == 0 {
            return Self::new();
        }

        let interval = UsizeCO::checked_from_start_len(0, end).unwrap();
        self.slice(interval)
    }
}

#[cfg(test)]
mod tests_for_slice;
