use alloc::vec::Vec;
use int_interval::UsizeCO;

use crate::WORD_BITS;

pub(crate) struct Bits;

impl Bits {
    #[inline]
    pub(crate) fn last_word_mask(len: usize) -> u64 {
        let rem = len % WORD_BITS;
        if rem == 0 {
            u64::MAX
        } else {
            (1u64 << rem) - 1
        }
    }

    #[inline]
    pub(crate) fn mask_unused(bits: &mut [u64], len: usize) {
        if let Some(last) = bits.last_mut() {
            *last &= Self::last_word_mask(len);
        }
    }

    #[inline]
    pub(crate) fn word_len(bit_len: usize) -> usize {
        bit_len / WORD_BITS + usize::from(bit_len % WORD_BITS != 0)
    }

    #[inline]
    pub(crate) fn zero_words(words: usize) -> Vec<u64> {
        let mut bits = Vec::with_capacity(words);
        bits.resize(words, 0);
        bits
    }

    #[inline]
    #[allow(dead_code)]
    pub(crate) fn shrink_words(bits: &[u64], words: usize) -> Vec<u64> {
        bits[..words].to_vec()
    }

    #[inline]
    pub(crate) fn read_a_bit_at(bits: &[u64], index: usize) -> bool {
        bits[index / WORD_BITS] & (1u64 << (index % WORD_BITS)) != 0
    }

    #[inline]
    pub(crate) fn set_a_bit_at(bits: &mut [u64], index: usize, value: bool) {
        let word = index / WORD_BITS;
        let mask = 1u64 << (index % WORD_BITS);

        if value {
            bits[word] |= mask;
        } else {
            bits[word] &= !mask;
        }
    }

    #[inline]
    pub(crate) fn low_mask(bits: usize) -> u64 {
        if bits == WORD_BITS {
            u64::MAX
        } else {
            (1u64 << bits) - 1
        }
    }

    #[inline]
    pub(crate) fn read_a_word_at(src: &[u64], bit_start: usize) -> u64 {
        let word = bit_start / WORD_BITS;
        let shift = bit_start % WORD_BITS;

        let lo = src.get(word).copied().unwrap_or(0) >> shift;

        if shift == 0 {
            lo
        } else {
            let hi = src.get(word + 1).copied().unwrap_or(0);
            lo | (hi << (WORD_BITS - shift))
        }
    }

    #[inline]
    pub(crate) fn write_a_word_at(dst: &mut [u64], bit_start: usize, value: u64, len: usize) {
        let value = value & Self::low_mask(len);
        let word = bit_start / WORD_BITS;
        let shift = bit_start % WORD_BITS;

        dst[word] |= value << shift;

        if shift != 0 && word + 1 < dst.len() {
            dst[word + 1] |= value >> (WORD_BITS - shift);
        }
    }

    /// Copies `len` bits from `src` (starting at `src_start`) to `dst`
    /// (starting at `dst_start`).
    ///
    /// When both offsets are word-aligned, whole `u64` words are copied in
    /// bulk via [`copy_from_slice`]; only the trailing partial word (if any)
    /// falls back to the per-chunk path.  Unaligned copies use the original
    /// chunk-by-chunk loop.
    #[inline]
    pub(crate) fn copy(
        src: &[u64],
        src_start: usize,
        dst: &mut [u64],
        dst_start: usize,
        len: usize,
    ) {
        // Fast path: both source and destination are word-aligned.
        if src_start.is_multiple_of(WORD_BITS) && dst_start.is_multiple_of(WORD_BITS) {
            let src_word = src_start / WORD_BITS;
            let dst_word = dst_start / WORD_BITS;
            let full_words = len / WORD_BITS;

            if full_words > 0 {
                dst[dst_word..dst_word + full_words]
                    .copy_from_slice(&src[src_word..src_word + full_words]);
            }

            let remainder_bits = len % WORD_BITS;
            if remainder_bits > 0 {
                let offset = full_words * WORD_BITS;
                let chunk = Self::read_a_word_at(src, src_start + offset);
                Self::write_a_word_at(dst, dst_start + offset, chunk, remainder_bits);
            }

            return;
        }

        // Slow path: unaligned copy, one chunk at a time.
        let mut done = 0usize;

        while done < len {
            let take = (len - done).min(WORD_BITS);
            let chunk = Self::read_a_word_at(src, src_start + done);
            Self::write_a_word_at(dst, dst_start + done, chunk, take);
            done += take;
        }
    }

    #[inline]
    pub(crate) fn assert_interval_in_bounds(interval: UsizeCO, len: usize) {
        assert!(
            interval.end_excl() <= len,
            "bit string interval out of bounds: {}..{}, len={}",
            interval.start(),
            interval.end_excl(),
            len
        );
    }

    /// Shifts `count` bits starting at `start` one position to the right
    /// (each source bit `i` → destination `i+1`), working in-place word by word.
    ///
    /// Bits below `start` are untouched.  The bit originally at
    /// `start + count - 1` lands at `start + count`; the bit at `start`
    /// is left as-is (caller must set it).
    #[inline]
    pub(crate) fn shift_right_in_place(bits: &mut [u64], start: usize, count: usize) {
        if count == 0 {
            return;
        }

        // Combined operating range: [start, start + count + 1).
        let dest_end = start + count + 1;
        let first_word = start / WORD_BITS;
        let last_word = (dest_end - 1) / WORD_BITS;
        let first_shift = start % WORD_BITS;
        let dest_end_shift = dest_end % WORD_BITS;

        let mut carry: u64 = 0;

        for w in first_word..=last_word {
            let cur = bits[w];
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

            let preserve_mask = Bits::low_mask(lo);
            let range_mask = Bits::low_mask(hi) & !Bits::low_mask(lo);
            let range = cur & range_mask;

            // The highest source bit in the range (at WORD_BITS-1) overflows
            // to the next word.  For the last word, this only happens when
            // hi == WORD_BITS (dest_end is word-aligned).
            let overflow = if hi == WORD_BITS {
                (range >> (WORD_BITS - 1)) & 1
            } else {
                0
            };

            // Shift left (i → i+1), OR carry from previous word at `lo`.
            let shifted = ((range << 1) & range_mask) | (carry << lo);

            bits[w] = (cur & preserve_mask) | shifted;
            carry = overflow;
        }
    }

    /// Shifts `count` bits starting at `start` one position to the left
    /// (each source bit `i` → destination `i-1`), working in-place word by word.
    ///
    /// The bit originally at `start - 1` is overwritten by the bit from
    /// `start`.  Bits at or above `start + count` are untouched.
    ///
    /// `start` must be >= 1 (so that `start - 1` does not underflow).
    #[inline]
    pub(crate) fn shift_left_in_place(bits: &mut [u64], start: usize, count: usize) {
        if count == 0 {
            return;
        }

        // Combined operating range: [start-1, start+count).
        let combined_start = start - 1;
        let end = start + count;
        let first_word = combined_start / WORD_BITS;
        let last_word = (end - 1) / WORD_BITS;
        let first_shift = combined_start % WORD_BITS;
        let end_shift = end % WORD_BITS;

        let mut carry: u64 = 0;

        // Process high-to-low: the LSB of each word's range overflows to
        // the previous word's MSB.
        for w in (first_word..=last_word).rev() {
            let cur = bits[w];
            let lo = if w == first_word { first_shift } else { 0 };
            let hi = if w == last_word {
                if end_shift == 0 { WORD_BITS } else { end_shift }
            } else {
                WORD_BITS
            };

            let preserve_mask = Bits::low_mask(lo) | !Bits::low_mask(hi);
            let range_mask = Bits::low_mask(hi) & !Bits::low_mask(lo);
            let range = cur & range_mask;

            // The bit at position `lo` is destination-only for the first
            // word of the combined range.  For non-first words (lo == 0),
            // the lowest bit overflows to the previous word's MSB.
            let overflow = if lo == 0 && w > first_word {
                range & 1
            } else {
                0
            };

            // Shift right (i → i-1), OR carry from next-higher word at
            // the highest position in this word's range.
            let shifted = if hi > 0 {
                ((range >> 1) & range_mask) | (carry << (hi - 1))
            } else {
                0
            };

            bits[w] = (cur & preserve_mask) | shifted;
            carry = overflow;
        }
    }

    /// Clears `len` bits in `words` starting at `bit_start`.
    ///
    /// `len` must be > 0.
    #[inline]
    pub(crate) fn clear_bits(words: &mut [u64], start: usize, len: usize) {
        debug_assert!(len > 0);
        let end = start + len;
        let first = start / WORD_BITS;
        let last = end.saturating_sub(1) / WORD_BITS;

        if first == last {
            let mask = Self::low_mask(len) << (start % WORD_BITS);
            words[first] &= !mask;
        } else {
            words[first] &= Self::low_mask(start % WORD_BITS);
            for w in (first + 1)..last {
                words[w] = 0;
            }
            let end_rem = end % WORD_BITS;
            if end_rem != 0 {
                words[last] &= !Self::low_mask(end_rem);
            } else {
                words[last] = 0;
            }
        }
    }
}

#[cfg(test)]
mod tests_for_assert_interval_in_bounds;
#[cfg(test)]
mod tests_for_bit_at;
#[cfg(test)]
mod tests_for_copy;
#[cfg(test)]
mod tests_for_last_word_mask;
#[cfg(test)]
mod tests_for_low_mask;
#[cfg(test)]
mod tests_for_mask_unused;
#[cfg(test)]
mod tests_for_read_chunk;
#[cfg(test)]
mod tests_for_set_bit;
#[cfg(test)]
mod tests_for_shrink_words;
#[cfg(test)]
mod tests_for_word_len;
#[cfg(test)]
mod tests_for_write_chunk;
#[cfg(test)]
mod tests_for_zero_words;
