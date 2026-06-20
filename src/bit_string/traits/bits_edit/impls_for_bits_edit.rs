use crate::{WORD_BITS, bit_string::traits::*};

impl BitsEdit for [u64] {
    /// Masks the last used word with
    /// [`BitsMask::last_word_mask(len)`](BitsMask::last_word_mask),
    /// and zeros any words beyond `word_len(len)`.
    ///
    /// When `self` already contains exactly `word_len(len)` words only the
    /// last word is touched; any surplus words are cleared entirely.
    #[inline]
    fn mask_unused_bits(&mut self, len: usize) {
        let used = word_len(len);
        for w in used..self.len() {
            self[w] = 0;
        }
        if let Some(last) = self.get_mut(used.wrapping_sub(1)) {
            *last &= <[u64]>::last_word_mask(len);
        }
    }

    /// Indexes into `self[word]` and tests the target bit with a mask.
    #[inline]
    fn read_bit_at(&self, index: usize) -> bool {
        self[index / WORD_BITS] & (1u64 << (index % WORD_BITS)) != 0
    }

    /// Computes the word index and bit mask, then sets or clears the target bit.
    #[inline]
    fn set_bit_at(&mut self, index: usize, value: bool) {
        let word = index / WORD_BITS;
        let mask = 1u64 << (index % WORD_BITS);

        if value {
            self[word] |= mask;
        } else {
            self[word] &= !mask;
        }
    }

    /// Reads the low word at `bit_start / WORD_BITS`, shifted down by the
    /// intra-word offset. When the read crosses a word boundary the high part
    /// is stitched in from the next word.
    #[inline]
    fn read_word_at(&self, bit_start: usize) -> u64 {
        let word = bit_start / WORD_BITS;
        let shift = bit_start % WORD_BITS;

        let lo = self.get(word).copied().unwrap_or(0) >> shift;

        if shift == 0 {
            lo
        } else {
            let hi = self.get(word + 1).copied().unwrap_or(0);
            lo | (hi << (WORD_BITS - shift))
        }
    }

    /// Masks `value` down to `len` bits via [`BitsMask::low_mask`], then ORs it into
    /// `self` at `bit_start`. When the write crosses a word boundary the high
    /// part spills into the next word.
    #[inline]
    fn write_word_at(&mut self, bit_start: usize, value: u64, len: usize) {
        let value = value & <[u64]>::low_mask(len);
        let word = bit_start / WORD_BITS;
        let shift = bit_start % WORD_BITS;

        self[word] |= value << shift;

        if shift != 0 && word + 1 < self.len() {
            self[word + 1] |= value >> (WORD_BITS - shift);
        }
    }

    /// Capture `len` bits from `self` at `start` to be pasted later via
    /// [`BitsCopied::paste_to`].
    #[inline]
    fn copy_bits(&self, start: usize, len: usize) -> BitsCopied<'_> {
        BitsCopied {
            src: self,
            src_start: start,
            len,
            aligned: start.is_multiple_of(WORD_BITS),
        }
    }

    /// Walks each word in `[start, start + count + 1)`, extracts the affected
    /// range, shifts it left by one (toward higher indices), and ORs the
    /// carry from the previous word into the low side. The vacated low bit
    /// in the range is zero-filled and the high bit that falls off becomes
    /// the carry to the next word.
    #[inline]
    fn shift_right_in_place(&mut self, start: usize, count: usize) {
        if count == 0 {
            return;
        }

        let dest_end = start + count + 1;
        let first_word = start / WORD_BITS;
        let last_word = (dest_end - 1) / WORD_BITS;
        let first_shift = start % WORD_BITS;
        let dest_end_shift = dest_end % WORD_BITS;

        let mut carry: u64 = 0;

        for w in first_word..=last_word {
            let cur = self[w];
            let lo = if w == first_word { first_shift } else { 0 };
            let hi = if w == last_word {
                if dest_end_shift == 0 {
                    WORD_BITS
                } else {
                    dest_end_shift
                }
            } else {
                WORD_BITS
            };

            let preserve_mask = <[u64]>::low_mask(lo);
            let range_mask = <[u64]>::low_mask(hi) & !<[u64]>::low_mask(lo);
            let range = cur & range_mask;

            let overflow = if hi == WORD_BITS {
                (range >> (WORD_BITS - 1)) & 1
            } else {
                0
            };

            let shifted = ((range << 1) & range_mask) | (carry << lo);

            self[w] = (cur & preserve_mask) | shifted;
            carry = overflow;
        }
    }

    /// Walks each word in `[start - 1, start + count)` in reverse, extracts the
    /// affected range, shifts it right by one (toward lower indices), and ORs
    /// the carry from the previous word into the high side. The vacated high
    /// bit in the range is zero-filled and the low bit that falls off becomes
    /// the carry to the previous word.
    #[inline]
    fn shift_left_in_place(&mut self, start: usize, count: usize) {
        if count == 0 {
            return;
        }

        let combined_start = start - 1;
        let end = start + count;
        let first_word = combined_start / WORD_BITS;
        let last_word = (end - 1) / WORD_BITS;
        let first_shift = combined_start % WORD_BITS;
        let end_shift = end % WORD_BITS;

        let mut carry: u64 = 0;

        for w in (first_word..=last_word).rev() {
            let cur = self[w];
            let lo = if w == first_word { first_shift } else { 0 };
            let hi = if w == last_word {
                if end_shift == 0 { WORD_BITS } else { end_shift }
            } else {
                WORD_BITS
            };

            let preserve_mask = <[u64]>::low_mask(lo) | !<[u64]>::low_mask(hi);
            let range_mask = <[u64]>::low_mask(hi) & !<[u64]>::low_mask(lo);
            let range = cur & range_mask;

            let overflow = if lo == 0 && w > first_word {
                range & 1
            } else {
                0
            };

            let shifted = if hi > 0 {
                ((range >> 1) & range_mask) | (carry << (hi - 1))
            } else {
                0
            };

            self[w] = (cur & preserve_mask) | shifted;
            carry = overflow;
        }
    }

    /// Clears `len` bits in `self` starting at `start`. `len` must be > 0.
    #[inline]
    fn clear_bits_at(&mut self, start: usize, len: usize) {
        debug_assert!(len > 0);
        let end = start + len;
        let first = start / WORD_BITS;
        let last = end.saturating_sub(1) / WORD_BITS;

        if first == last {
            let mask = <[u64]>::low_mask(len) << (start % WORD_BITS);
            self[first] &= !mask;
        } else {
            self[first] &= <[u64]>::low_mask(start % WORD_BITS);
            for w in (first + 1)..last {
                self[w] = 0;
            }
            let end_rem = end % WORD_BITS;
            if end_rem != 0 {
                self[last] &= !<[u64]>::low_mask(end_rem);
            } else {
                self[last] = 0;
            }
        }
    }
}

#[cfg(test)]
mod tests_for_bit_at;

#[cfg(test)]
mod tests_for_mask_unused;

#[cfg(test)]
mod tests_for_read_chunk;

#[cfg(test)]
mod tests_for_set_bit;

#[cfg(test)]
mod tests_for_write_chunk;
