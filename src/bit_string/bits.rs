use alloc::vec::Vec;
use int_interval::UsizeCO;

use crate::WORD_BITS;

// ---------------------------------------------------------------------------
// Free functions (pure math / constructors, no [u64] receiver)
// ---------------------------------------------------------------------------

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
pub(crate) fn low_mask(bits: usize) -> u64 {
    if bits == WORD_BITS {
        u64::MAX
    } else {
        (1u64 << bits) - 1
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

// ---------------------------------------------------------------------------
// CopyFrom — zero-cost curried copy: source snapshot + deferred paste
// ---------------------------------------------------------------------------

pub(crate) struct CopyFrom<'a> {
    src: &'a [u64],
    src_start: usize,
    len: usize,
    aligned: bool, // src_start % WORD_BITS == 0
}

impl CopyFrom<'_> {
    /// Paste the previously captured source bits into `dst` at `dst_start`.
    ///
    /// Fast path when both source and destination are word-aligned; otherwise
    /// falls back to a chunk-by-chunk read/write loop.
    #[inline]
    pub(crate) fn paste_to(&self, dst: &mut [u64], dst_start: usize) {
        if self.aligned && dst_start.is_multiple_of(WORD_BITS) {
            let src_word = self.src_start / WORD_BITS;
            let dst_word = dst_start / WORD_BITS;
            let full_words = self.len / WORD_BITS;

            if full_words > 0 {
                dst[dst_word..dst_word + full_words]
                    .copy_from_slice(&self.src[src_word..src_word + full_words]);
            }

            let remainder_bits = self.len % WORD_BITS;
            if remainder_bits > 0 {
                let offset = full_words * WORD_BITS;
                let chunk = self.src.read_word_at(self.src_start + offset);
                dst.write_word_at(dst_start + offset, chunk, remainder_bits);
            }
            return;
        }

        let mut done = 0usize;
        while done < self.len {
            let take = (self.len - done).min(WORD_BITS);
            let chunk = self.src.read_word_at(self.src_start + done);
            dst.write_word_at(dst_start + done, chunk, take);
            done += take;
        }
    }
}

// ---------------------------------------------------------------------------
// Bits trait — extends [u64] with bit-level operations
// ---------------------------------------------------------------------------

pub(crate) trait Bits {
    fn mask_unused_bits(&mut self, len: usize);
    fn read_bit_at(&self, index: usize) -> bool;
    fn set_bit_at(&mut self, index: usize, value: bool);
    fn read_word_at(&self, bit_start: usize) -> u64;
    fn write_word_at(&mut self, bit_start: usize, value: u64, len: usize);
    fn copy_from(&self, start: usize, len: usize) -> CopyFrom<'_>;
    fn clear_bits_at(&mut self, start: usize, len: usize);
    fn shift_right_in_place(&mut self, start: usize, count: usize);
    fn shift_left_in_place(&mut self, start: usize, count: usize);
}

impl Bits for [u64] {
    #[inline]
    fn mask_unused_bits(&mut self, len: usize) {
        if let Some(last) = self.last_mut() {
            *last &= last_word_mask(len);
        }
    }

    #[inline]
    fn read_bit_at(&self, index: usize) -> bool {
        self[index / WORD_BITS] & (1u64 << (index % WORD_BITS)) != 0
    }

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

    #[inline]
    fn write_word_at(&mut self, bit_start: usize, value: u64, len: usize) {
        let value = value & low_mask(len);
        let word = bit_start / WORD_BITS;
        let shift = bit_start % WORD_BITS;

        self[word] |= value << shift;

        if shift != 0 && word + 1 < self.len() {
            self[word + 1] |= value >> (WORD_BITS - shift);
        }
    }

    /// Capture `len` bits from `self` at `start` to be pasted later via
    /// [`CopyFrom::paste_to`].
    #[inline]
    fn copy_from(&self, start: usize, len: usize) -> CopyFrom<'_> {
        CopyFrom {
            src: self,
            src_start: start,
            len,
            aligned: start.is_multiple_of(WORD_BITS),
        }
    }

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

            let preserve_mask = low_mask(lo);
            let range_mask = low_mask(hi) & !low_mask(lo);
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

            let preserve_mask = low_mask(lo) | !low_mask(hi);
            let range_mask = low_mask(hi) & !low_mask(lo);
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
            let mask = low_mask(len) << (start % WORD_BITS);
            self[first] &= !mask;
        } else {
            self[first] &= low_mask(start % WORD_BITS);
            for w in (first + 1)..last {
                self[w] = 0;
            }
            let end_rem = end % WORD_BITS;
            if end_rem != 0 {
                self[last] &= !low_mask(end_rem);
            } else {
                self[last] = 0;
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
