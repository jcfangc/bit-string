use int_interval::UsizeCO;

use crate::funcs_for_bits::*;
use crate::traits::*;

use super::*;

impl BitString {
    /// Clamp an interval to `[0, self.bit_len())`. Returns `None` when the
    /// clamped interval is empty.
    #[inline]
    fn clamp_drain_interval(&self, interval: UsizeCO) -> Option<UsizeCO> {
        let start = interval.start().min(self.bit_len);
        let end = interval.end_excl().min(self.bit_len).max(start);
        UsizeCO::try_new(start, end)
    }

    /// Allocate a new buffer with `clamped` removed from `self`.
    #[inline]
    fn drain_allocate(&self, clamped: UsizeCO) -> (Vec<u64>, usize) {
        let removed_len = clamped.len();
        let tail_len = self.bit_len - clamped.end_excl();
        let new_len = self.bit_len - removed_len;

        let mut dst = zero_words(word_len(new_len));
        self.words
            .copy_bits(0, clamped.start())
            .paste_to(&mut dst, 0);
        self.words
            .copy_bits(clamped.end_excl(), tail_len)
            .paste_to(&mut dst, clamped.start());
        (dst, new_len)
    }
}

// ---------------------------------------------------------------------------
// drain_interval  variants
// ---------------------------------------------------------------------------

impl BitString {
    /// Borrowing variant: returns a new [`BitString`] with `interval` removed;
    /// `self` is unchanged.
    ///
    /// The interval is clamped to `[0, self.bit_len())`.  An interval that lies
    /// entirely beyond the bit string length returns a clone of `self`.
    #[inline]
    pub fn drain_interval(&self, interval: UsizeCO) -> Self {
        let Some(clamped) = self.clamp_drain_interval(interval) else {
            return self.clone();
        };
        let (words, bit_len) = self.drain_allocate(clamped);
        BitString { words, bit_len }
    }

    /// Assigning variant: removes `interval` from `self`.
    ///
    /// Always allocates a fresh buffer and swaps it in — profiling shows that
    /// `copy_bits → paste_to` via memcpy beats any in-place bit-shifting
    /// approach for typical workloads.
    #[inline]
    pub fn drain_interval_assign(&mut self, interval: UsizeCO) {
        let Some(clamped) = self.clamp_drain_interval(interval) else {
            return;
        };
        let (words, bit_len) = self.drain_allocate(clamped);
        self.words = words;
        self.bit_len = bit_len;
    }
}

#[cfg(test)]
mod tests_for_drain_interval;
