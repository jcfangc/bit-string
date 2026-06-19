use int_interval::UsizeCO;

use crate::bit_string::bits::*;

use super::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

impl BitString {
    /// Clamp an interval to `[0, self.bit_len()]`. Returns `(start, end)` with
    /// `end >= start`.
    #[inline]
    fn clamp_replace_interval(&self, interval: UsizeCO) -> (usize, usize) {
        let start = interval.start().min(self.bit_len);
        let end = interval.end_excl().min(self.bit_len).max(start);
        (start, end)
    }

    /// Allocate a new buffer with the pre-clamped interval replaced by
    /// `replacement`.
    ///
    /// `start` and `end` must be pre-clamped via
    /// [`clamp_replace_interval`](Self::clamp_replace_interval).
    #[inline]
    fn replace_allocate(&self, start: usize, end: usize, replacement: &Self) -> BitString {
        let repl_len = replacement.bit_len;
        let tail_len = self.bit_len - end;
        let new_len = start
            .checked_add(repl_len)
            .and_then(|n| n.checked_add(tail_len))
            .expect("bit string length overflow");

        let mut dst = zero_words(word_len(new_len));
        self.words.copy_bits(0, start).paste_to(&mut dst, 0);
        replacement
            .words
            .copy_bits(0, repl_len)
            .paste_to(&mut dst, start);
        self.words
            .copy_bits(end, tail_len)
            .paste_to(&mut dst, start + repl_len);

        BitString {
            words: dst,
            bit_len: new_len,
        }
    }

    /// In-place overwrite: clear `start..start+replacement.bit_len()` then copy
    /// `replacement` into the cleared region.
    ///
    /// Caller guarantees `replacement.bit_len() == (end - start)` — i.e. the
    /// replacement length equals the clamped interval length, so the bit string
    /// length does not change.
    #[inline]
    fn replace_equal_length_in_place(&mut self, start: usize, replacement: &Self) {
        let repl_len = replacement.bit_len;
        if repl_len == 0 {
            return;
        }
        self.words.clear_bits_at(start, repl_len);
        replacement
            .words
            .copy_bits(0, repl_len)
            .paste_to(&mut self.words, start);
    }

    /// Compute the clamped `(start, end)` range for the
    /// [`replace`](Self::replace) convenience methods from a start position and
    /// replacement length.
    #[inline]
    fn clamp_replace_range(&self, start: usize, len: usize) -> (usize, usize) {
        let start = start.min(self.bit_len);
        let end = self.bit_len.min(start.saturating_add(len));
        (start, end)
    }
}

// ---------------------------------------------------------------------------
// replace_interval  variants
// ---------------------------------------------------------------------------

impl BitString {
    /// Borrowing variant: returns a new [`BitString`]; `self` is unchanged.
    #[inline]
    pub fn replace_interval(&self, interval: UsizeCO, replacement: &Self) -> Self {
        let (start, end) = self.clamp_replace_interval(interval);
        self.replace_allocate(start, end, replacement)
    }

    /// Assigning variant: replaces the interval in-place.
    ///
    /// When the replacement length equals the (clamped) interval length the
    /// operation is performed in-place without allocation.  Otherwise a fresh
    /// buffer is allocated and swapped in.
    pub fn replace_interval_assign(&mut self, interval: UsizeCO, replacement: &Self) {
        let (start, end) = self.clamp_replace_interval(interval);

        if replacement.bit_len == end - start {
            self.replace_equal_length_in_place(start, replacement);
            return;
        }

        let result = self.replace_allocate(start, end, replacement);
        self.words = result.words;
        self.bit_len = result.bit_len;
    }

    /// Consuming variant: replaces the interval, reusing `self`'s allocation
    /// when the replacement has the same length as the clamped interval.
    #[inline]
    pub fn replace_interval_into(mut self, interval: UsizeCO, replacement: &Self) -> Self {
        let (start, end) = self.clamp_replace_interval(interval);

        if replacement.bit_len == end - start {
            self.replace_equal_length_in_place(start, replacement);
            return self;
        }

        self.replace_allocate(start, end, replacement)
    }
}

// ---------------------------------------------------------------------------
// replace  convenience variants
// ---------------------------------------------------------------------------

impl BitString {
    /// Borrowing variant of [`replace`](Self::replace).
    #[inline]
    pub fn replace(&self, start: usize, replacement: &Self) -> Self {
        let (start, end) = self.clamp_replace_range(start, replacement.bit_len);
        if start == end {
            let mut result = self.clone();
            result.insert_bit_string(start, replacement);
            return result;
        }
        // SAFETY: start < end by the guard above.
        let interval = unsafe { UsizeCO::new_unchecked(start, end) };
        self.replace_interval(interval, replacement)
    }

    /// Assigning variant of [`replace`](Self::replace).
    #[inline]
    pub fn replace_assign(&mut self, start: usize, replacement: &Self) {
        let (start, end) = self.clamp_replace_range(start, replacement.bit_len);
        if start == end {
            self.insert_bit_string(start, replacement);
            return;
        }
        // SAFETY: start < end by the guard above.
        let interval = unsafe { UsizeCO::new_unchecked(start, end) };
        self.replace_interval_assign(interval, replacement);
    }

    /// Consuming variant of [`replace`](Self::replace).
    #[inline]
    pub fn replace_into(self, start: usize, replacement: &Self) -> Self {
        let (start, end) = self.clamp_replace_range(start, replacement.bit_len);
        if start == end {
            let mut this = self;
            this.insert_bit_string(start, replacement);
            return this;
        }
        // SAFETY: start < end by the guard above.
        let interval = unsafe { UsizeCO::new_unchecked(start, end) };
        self.replace_interval_into(interval, replacement)
    }
}

#[cfg(test)]
mod tests_for_replace;
#[cfg(test)]
mod tests_for_replace_interval;
