use int_interval::UsizeCO;

use crate::bit_string::bits::*;

use super::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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

    /// In-place shift-left of the tail after removing `clamped`.
    ///
    /// Precondition: `clamped.len() >= WORD_BITS` and the tail is non-empty
    /// (gap between src and dst ≥ 1 word, so read/write never alias within
    /// the same u64).
    fn drain_shift_in_place(&mut self, clamped: UsizeCO) {
        debug_assert!(clamped.len() >= WORD_BITS);

        let end = clamped.end_excl();
        let start = clamped.start();
        let tail_len = self.bit_len - end;

        let mut offset = 0usize;
        while offset < tail_len {
            let take = WORD_BITS.min(tail_len - offset);
            let chunk = self.words.read_word_at(end + offset);
            self.words.clear_bits_at(start + offset, take);
            self.words.write_word_at(start + offset, chunk, take);
            offset += take;
        }

        let new_len = self.bit_len - clamped.len();
        let new_words = word_len(new_len);
        self.words.truncate(new_words);
        if self.words.capacity() > new_words * 2 {
            self.words.shrink_to(new_words);
        }
        self.words.mask_unused_bits(new_len);
        self.bit_len = new_len;
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

    /// Assigning variant: removes `interval` from `self` in-place.
    ///
    /// The interval is clamped to `[0, self.bit_len())`.  When the clamped gap
    /// is at least one word (`removed_len >= 64`) the shift is performed
    /// in-place without allocation.  Otherwise a fresh buffer is allocated and
    /// swapped in.
    pub fn drain_interval_assign(&mut self, interval: UsizeCO) {
        let Some(clamped) = self.clamp_drain_interval(interval) else {
            return;
        };
        if clamped.len() >= WORD_BITS && clamped.end_excl() < self.bit_len {
            self.drain_shift_in_place(clamped);
            return;
        }
        let (words, bit_len) = self.drain_allocate(clamped);
        self.words = words;
        self.bit_len = bit_len;
    }

    /// Consuming variant: removes `interval`, reusing `self`'s allocation
    /// when the clamped gap is at least one word.
    ///
    /// The interval is clamped to `[0, self.bit_len())`.  An interval that lies
    /// entirely beyond the bit string length returns `self` unchanged.
    #[inline]
    pub fn drain_interval_into(mut self, interval: UsizeCO) -> Self {
        let Some(clamped) = self.clamp_drain_interval(interval) else {
            return self;
        };
        if clamped.len() >= WORD_BITS && clamped.end_excl() < self.bit_len {
            self.drain_shift_in_place(clamped);
            return self;
        }
        let (words, bit_len) = self.drain_allocate(clamped);
        BitString { words, bit_len }
    }
}

#[cfg(test)]
mod tests_for_drain_interval;
