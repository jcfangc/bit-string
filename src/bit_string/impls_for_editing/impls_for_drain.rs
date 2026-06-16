use int_interval::UsizeCO;

use crate::bit_string::bits::Bits;

use super::*;

// ---------------------------------------------------------------------------
// Core helper
// ---------------------------------------------------------------------------

/// Shared allocation: produces a new [`BitString`] from `src` with `interval`
/// removed.  The interval must be in bounds.
#[inline]
fn drain_interval_core(src: &[u64], src_len: usize, interval: UsizeCO) -> BitString {
    let start = interval.start();
    let end = interval.end_excl();
    let removed_len = interval.len();
    let tail_len = src_len - end;
    let new_len = src_len - removed_len;

    let mut dst = Bits::zero_words(Bits::word_len(new_len));
    Bits::copy(src, 0, &mut dst, 0, start);
    Bits::copy(src, end, &mut dst, start, tail_len);

    BitString {
        words: dst,
        bit_len: new_len,
    }
}

/// In-place shift-left for drain: copies `tail_len` bits from `end` to
/// `start`.  Panics if `removed_len < WORD_BITS` (gap too small for safe
/// in-place copy).
#[inline]
fn drain_shift_in_place(
    words: &mut Vec<u64>,
    bit_len: &mut usize,
    start: usize,
    end: usize,
    removed_len: usize,
    tail_len: usize,
) {
    debug_assert!(removed_len >= WORD_BITS && tail_len > 0);

    // Read chunks from the tail (high to low is safe for shift-left when
    // dst < src and gap >= WORD_BITS, but we process low-to-high with
    // read-then-write to avoid &/&mut overlap).
    let mut offset = 0usize;
    while offset < tail_len {
        let take = WORD_BITS.min(tail_len - offset);
        let chunk = Bits::read_chunk(words, end + offset);
        Bits::clear_bits(words, start + offset, take);
        Bits::write_chunk(words, start + offset, chunk, take);
        offset += take;
    }

    let new_len = *bit_len - removed_len;
    let new_words = Bits::word_len(new_len);
    words.truncate(new_words);
    if words.capacity() > new_words * 2 {
        words.shrink_to(new_words);
    }
    Bits::mask_unused(words, new_len);
    *bit_len = new_len;
}

// ---------------------------------------------------------------------------
// drain_interval  variants
// ---------------------------------------------------------------------------

impl BitString {
    /// Borrowing variant: returns a new [`BitString`] with `interval` removed;
    /// `self` is unchanged.
    #[inline]
    pub fn drain_interval(&self, interval: UsizeCO) -> Self {
        Bits::assert_interval_in_bounds(interval, self.bit_len);
        drain_interval_core(&self.words, self.bit_len, interval)
    }

    /// Assigning variant: removes `interval` from `self` in-place.
    ///
    /// When the gap between the tail and its destination is at least one word
    /// (`interval.len() >= 64`) the shift is performed in-place without
    /// allocation.  Otherwise a fresh buffer is allocated and swapped in.
    pub fn drain_interval_assign(&mut self, interval: UsizeCO) {
        Bits::assert_interval_in_bounds(interval, self.bit_len);

        let start = interval.start();
        let end = interval.end_excl();
        let removed_len = interval.len();
        if removed_len == 0 {
            return;
        }

        let tail_len = self.bit_len - end;

        if removed_len >= WORD_BITS && tail_len > 0 {
            drain_shift_in_place(
                &mut self.words,
                &mut self.bit_len,
                start,
                end,
                removed_len,
                tail_len,
            );
            return;
        }

        // Fallback: allocate fresh buffer and swap.
        let result = drain_interval_core(&self.words, self.bit_len, interval);
        self.words = result.words;
        self.bit_len = result.bit_len;
    }

    /// Consuming variant: removes `interval`, reusing `self`'s allocation
    /// when the gap is at least one word.
    #[inline]
    pub fn drain_interval_into(mut self, interval: UsizeCO) -> Self {
        Bits::assert_interval_in_bounds(interval, self.bit_len);

        let start = interval.start();
        let end = interval.end_excl();
        let removed_len = interval.len();
        if removed_len == 0 {
            return self;
        }

        let tail_len = self.bit_len - end;

        if removed_len >= WORD_BITS && tail_len > 0 {
            drain_shift_in_place(
                &mut self.words,
                &mut self.bit_len,
                start,
                end,
                removed_len,
                tail_len,
            );
            return self;
        }

        drain_interval_core(&self.words, self.bit_len, interval)
    }
}

#[cfg(test)]
mod tests_for_drain_interval;
