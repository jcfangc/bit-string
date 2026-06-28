use crate::traits::*;
use crate::{WORD_BITS, low_mask};

use crate::BitStr;

// ---------------------------------------------------------------------------
// Trailing helper — scans from the end backwards
// ---------------------------------------------------------------------------

/// Counts consecutive bits equal to the fill value from the end of a view.
///
/// `FILL_ONES` selects the fill value: `false` → zeros, `true` → ones.
#[inline]
fn trailing_value_count<const FILL_ONES: bool>(
    words: &[u64],
    start: usize,
    bit_len: usize,
) -> usize {
    let end = start + bit_len;
    let start_offset = start % WORD_BITS;
    let end_rem = end % WORD_BITS;
    let last_wi = (end - 1) / WORD_BITS;
    let start_wi = start / WORD_BITS;

    let mut scanned = 0usize;

    // Last word (partial, if end_rem != 0).
    if end_rem != 0 {
        let last_limit = if last_wi == start_wi {
            end_rem - start_offset
        } else {
            end_rem
        };
        let last_val = if last_wi == start_wi {
            words[last_wi] >> start_offset
        } else {
            words[last_wi] & low_mask(end_rem)
        };
        let last_count = count_leading_within::<FILL_ONES>(last_val, last_limit);
        if last_count < last_limit {
            return last_count;
        }
        scanned += last_limit;

        // Entire view was within a single partial word.
        if last_wi == start_wi {
            return scanned.min(bit_len);
        }
    }

    // Full middle words — SIMD-accelerated, from right to left.
    let wi_end = if end_rem != 0 { last_wi - 1 } else { last_wi };
    let mid_first = if start_offset > 0 {
        start_wi + 1
    } else {
        start_wi
    };
    if wi_end >= mid_first {
        let nr_words = wi_end + 1 - mid_first;
        let trailing_w = if FILL_ONES {
            words[mid_first..=wi_end].trailing_one_words()
        } else {
            words[mid_first..=wi_end].trailing_zero_words()
        };
        if trailing_w < nr_words {
            let hit_wi = wi_end - trailing_w;
            scanned += trailing_w * WORD_BITS;
            return scanned + count_leading::<FILL_ONES>(words[hit_wi]).min(WORD_BITS);
        }
        scanned += trailing_w * WORD_BITS;
    }

    // First word (partial, if start_offset != 0 and not already handled).
    if start_offset > 0 {
        let first_limit = WORD_BITS - start_offset;
        let first_val = words[start_wi] >> start_offset;
        let first_count = count_leading_within::<FILL_ONES>(first_val, first_limit);
        scanned += first_count.min(first_limit);
    }

    scanned.min(bit_len)
}

/// Counts leading bits of a given value within a full u64 word.
///
/// `FILL_ONES = false` → leading zeros, `FILL_ONES = true` → leading ones.
#[inline]
fn count_leading<const FILL_ONES: bool>(val: u64) -> usize {
    if FILL_ONES {
        (!val).leading_zeros() as usize
    } else {
        val.leading_zeros() as usize
    }
}

/// Counts leading bits of a value within its highest `limit` bits.
///
/// Shifts valid bits to the top of the u64 so that [`u64::leading_zeros`]
/// counts from the highest valid bit downwards.
#[inline]
fn count_leading_within<const FILL_ONES: bool>(val: u64, limit: usize) -> usize {
    if limit == 0 {
        return 0;
    }
    let shifted = val << (WORD_BITS - limit);
    if FILL_ONES {
        ((!shifted).leading_zeros() as usize).min(limit)
    } else {
        (shifted.leading_zeros() as usize).min(limit)
    }
}

impl<'bs> BitStr<'bs> {
    /// Returns the number of consecutive `false` bits from the **end** of this
    /// view.
    ///
    /// # Examples
    ///
    /// ```
    /// use bit_string::BitString;
    ///
    /// let bits = BitString::try_from("10000").unwrap();
    /// let v = bits.as_bit_str();
    /// assert_eq!(v.trailing_zeros(), 4);
    /// ```
    #[inline]
    pub fn trailing_zeros(&self) -> usize {
        if self.bit_len == 0 {
            return 0;
        }
        trailing_value_count::<false>(self.source.words(), self.start, self.bit_len)
    }

    /// Returns the number of consecutive `true` bits from the **end** of this
    /// view.
    ///
    /// # Examples
    ///
    /// ```
    /// use bit_string::BitString;
    ///
    /// let bits = BitString::try_from("01111").unwrap();
    /// let v = bits.as_bit_str();
    /// assert_eq!(v.trailing_ones(), 4);
    /// ```
    #[inline]
    pub fn trailing_ones(&self) -> usize {
        if self.bit_len == 0 {
            return 0;
        }
        trailing_value_count::<true>(self.source.words(), self.start, self.bit_len)
    }
}

#[cfg(test)]
mod tests_for_trailing_zeros;

#[cfg(test)]
mod tests_for_trailing_ones;
