use int_interval::UsizeCO;

use crate::bit_string::bits::*;

use super::*;

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
        let start = interval.start().min(self.bit_len);
        let end = interval.end_excl().min(self.bit_len).max(start);
        let Some(clamped) = UsizeCO::try_new(start, end) else {
            return self.clone();
        };

        let removed_len = clamped.len();
        let tail_len = self.bit_len - clamped.end_excl();
        let new_len = self.bit_len - removed_len;

        let mut dst = zero_words(word_len(new_len));
        self.words.copy_bits_to(0, &mut dst, 0, clamped.start());
        self.words
            .copy_bits_to(clamped.end_excl(), &mut dst, clamped.start(), tail_len);

        BitString {
            words: dst,
            bit_len: new_len,
        }
    }

    /// Assigning variant: removes `interval` from `self` in-place.
    ///
    /// The interval is clamped to `[0, self.bit_len())`.  When the clamped gap
    /// is at least one word (`removed_len >= 64`) the shift is performed
    /// in-place without allocation.  Otherwise a fresh buffer is allocated and
    /// swapped in.
    pub fn drain_interval_assign(&mut self, interval: UsizeCO) {
        let start = interval.start().min(self.bit_len);
        let end = interval.end_excl().min(self.bit_len).max(start);
        let Some(clamped) = UsizeCO::try_new(start, end) else {
            return;
        };

        let removed_len = clamped.len();
        let tail_len = self.bit_len - clamped.end_excl();

        // In-place fast path: when the gap left by the removed interval is at
        // least one word wide, we can shift the tail leftward without allocating.
        //
        // Preconditions:
        //   removed_len >= WORD_BITS  → gap between src and dst is ≥ 1 word,
        //       so read/write never overlap within the same u64.
        //   tail_len > 0              → there are bits after the removed interval
        //       that need to be shifted.
        if removed_len >= WORD_BITS && tail_len > 0 {
            let end = clamped.end_excl();
            let start = clamped.start();

            // Copy the tail word by word from [end, bit_len) to [start, ...).
            // Processed in WORD_BITS-sized chunks from low to high address.
            // read_chunk + clear_bits + write_chunk is used instead of copy
            // because src and dst may alias across different words within the
            // same Vec.
            let mut offset = 0usize;
            while offset < tail_len {
                let take = WORD_BITS.min(tail_len - offset);
                let chunk = self.words.read_word_at(end + offset);
                self.words.clear_bits_at(start + offset, take);
                self.words.write_word_at(start + offset, chunk, take);
                offset += take;
            }

            // Truncate the word array to the new length, shrink if overallocated,
            // then mask the unused bits in the last word.
            let new_len = self.bit_len - removed_len;
            let new_words = word_len(new_len);
            self.words.truncate(new_words);
            if self.words.capacity() > new_words * 2 {
                self.words.shrink_to(new_words);
            }
            self.words.mask_unused_bits(new_len);
            self.bit_len = new_len;
            return;
        }

        // Fallback: allocate fresh buffer and swap.
        let new_len = self.bit_len - removed_len;
        let mut dst = zero_words(word_len(new_len));
        self.words.copy_bits_to(0, &mut dst, 0, clamped.start());
        self.words
            .copy_bits_to(clamped.end_excl(), &mut dst, clamped.start(), tail_len);
        self.words = dst;
        self.bit_len = new_len;
    }

    /// Consuming variant: removes `interval`, reusing `self`'s allocation
    /// when the clamped gap is at least one word.
    ///
    /// The interval is clamped to `[0, self.bit_len())`.  An interval that lies
    /// entirely beyond the bit string length returns `self` unchanged.
    #[inline]
    pub fn drain_interval_into(mut self, interval: UsizeCO) -> Self {
        let start = interval.start().min(self.bit_len);
        let end = interval.end_excl().min(self.bit_len).max(start);
        let Some(clamped) = UsizeCO::try_new(start, end) else {
            return self;
        };

        let removed_len = clamped.len();
        let tail_len = self.bit_len - clamped.end_excl();

        // In-place fast path: when the gap left by the removed interval is at
        // least one word wide, we can shift the tail leftward without allocating.
        //
        // Preconditions:
        //   removed_len >= WORD_BITS  → gap between src and dst is ≥ 1 word,
        //       so read/write never overlap within the same u64.
        //   tail_len > 0              → there are bits after the removed interval
        //       that need to be shifted.
        if removed_len >= WORD_BITS && tail_len > 0 {
            let end = clamped.end_excl();
            let start = clamped.start();

            // Copy the tail word by word from [end, bit_len) to [start, ...).
            // Processed in WORD_BITS-sized chunks from low to high address.
            // read_chunk + clear_bits + write_chunk is used instead of copy
            // because src and dst may alias across different words within the
            // same Vec.
            let mut offset = 0usize;
            while offset < tail_len {
                let take = WORD_BITS.min(tail_len - offset);
                let chunk = self.words.read_word_at(end + offset);
                self.words.clear_bits_at(start + offset, take);
                self.words.write_word_at(start + offset, chunk, take);
                offset += take;
            }

            // Truncate the word array to the new length, shrink if overallocated,
            // then mask the unused bits in the last word.
            let new_len = self.bit_len - removed_len;
            let new_words = word_len(new_len);
            self.words.truncate(new_words);
            if self.words.capacity() > new_words * 2 {
                self.words.shrink_to(new_words);
            }
            self.words.mask_unused_bits(new_len);
            self.bit_len = new_len;
            return self;
        }

        // Fallback: allocate fresh buffer.
        let new_len = self.bit_len - removed_len;
        let mut dst = zero_words(word_len(new_len));
        self.words.copy_bits_to(0, &mut dst, 0, clamped.start());
        self.words
            .copy_bits_to(clamped.end_excl(), &mut dst, clamped.start(), tail_len);
        BitString {
            words: dst,
            bit_len: new_len,
        }
    }
}

#[cfg(test)]
mod tests_for_drain_interval;
