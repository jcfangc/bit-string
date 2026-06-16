use int_interval::UsizeCO;

use crate::bit_string::bits::Bits;

use super::*;

impl BitString {
    pub fn slice(&self, interval: UsizeCO) -> Self {
        Bits::assert_interval_in_bounds(interval, self.bit_len);

        let start = interval.start();
        let len = interval.len();

        let mut bits = Bits::zero_words(Bits::word_len(len));
        Bits::copy(&self.words, start, &mut bits, 0, len);

        Self {
            words: bits,
            bit_len: len,
        }
    }

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
