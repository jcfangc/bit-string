use int_interval::UsizeCO;

use crate::bit_string::bits::*;

use super::*;

// ---------------------------------------------------------------------------
// Core helper
// ---------------------------------------------------------------------------

/// Shared allocation: produces a new [`BitString`] from `src` by replacing
/// `interval` with `replacement`.  The interval is clamped to `[0, src_len]`.
#[inline]
fn replace_interval_core(
    src: &[u64],
    src_len: usize,
    interval: UsizeCO,
    replacement: &BitString,
) -> BitString {
    let start = interval.start().min(src_len);
    let end = interval.end_excl().min(src_len).max(start);
    let repl_len = replacement.bit_len;
    let tail_len = src_len - end;
    let new_len = start
        .checked_add(repl_len)
        .and_then(|n| n.checked_add(tail_len))
        .expect("bit string length overflow");

    let mut dst = zero_words(word_len(new_len));
    src.copy_bits(0, start).paste_to(&mut dst, 0);
    replacement
        .words
        .copy_bits(0, repl_len)
        .paste_to(&mut dst, start);
    src.copy_bits(end, tail_len)
        .paste_to(&mut dst, start + repl_len);

    BitString {
        words: dst,
        bit_len: new_len,
    }
}

// ---------------------------------------------------------------------------
// replace_interval  variants
// ---------------------------------------------------------------------------

impl BitString {
    /// Borrowing variant: returns a new [`BitString`]; `self` is unchanged.
    #[inline]
    pub fn replace_interval(&self, interval: UsizeCO, replacement: &Self) -> Self {
        replace_interval_core(&self.words, self.bit_len, interval, replacement)
    }

    /// Assigning variant: replaces the interval in-place.
    ///
    /// When the replacement length equals the (clamped) interval length the
    /// operation is performed in-place without allocation.  Otherwise a fresh
    /// buffer is allocated and swapped in.
    pub fn replace_interval_assign(&mut self, interval: UsizeCO, replacement: &Self) {
        let start = interval.start().min(self.bit_len);
        let end = interval.end_excl().min(self.bit_len).max(start);

        // Fast path: equal length — overwrite in-place.
        if replacement.bit_len == end - start {
            if replacement.bit_len > 0 {
                self.words.clear_bits_at(start, replacement.bit_len);
                replacement
                    .words
                    .copy_bits(0, replacement.bit_len)
                    .paste_to(&mut self.words, start);
            }
            return;
        }

        let result = replace_interval_core(&self.words, self.bit_len, interval, replacement);
        self.words = result.words;
        self.bit_len = result.bit_len;
    }

    /// Consuming variant: replaces the interval, reusing `self`'s allocation
    /// when the replacement has the same length as the clamped interval.
    #[inline]
    pub fn replace_interval_into(mut self, interval: UsizeCO, replacement: &Self) -> Self {
        let start = interval.start().min(self.bit_len);
        let end = interval.end_excl().min(self.bit_len).max(start);

        // Fast path: equal length — modify in-place, return self.
        if replacement.bit_len == end - start {
            if replacement.bit_len > 0 {
                self.words.clear_bits_at(start, replacement.bit_len);
                replacement
                    .words
                    .copy_bits(0, replacement.bit_len)
                    .paste_to(&mut self.words, start);
            }
            return self;
        }

        replace_interval_core(&self.words, self.bit_len, interval, replacement)
    }
}

// ---------------------------------------------------------------------------
// replace  convenience variants
// ---------------------------------------------------------------------------

impl BitString {
    /// Borrowing variant of [`replace`](Self::replace).
    #[inline]
    pub fn replace(&self, start: usize, replacement: &Self) -> Self {
        let start = start.min(self.bit_len);
        let end = self.bit_len.min(start.saturating_add(replacement.bit_len));
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
        let start = start.min(self.bit_len);
        let end = self.bit_len.min(start.saturating_add(replacement.bit_len));
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
        let bit_len = self.bit_len;
        let start = start.min(bit_len);
        let end = bit_len.min(start.saturating_add(replacement.bit_len));
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
