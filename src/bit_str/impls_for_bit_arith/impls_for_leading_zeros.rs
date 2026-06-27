use crate::traits::*;
use crate::{WORD_BITS, low_mask};

use crate::BitStr;

// ---------------------------------------------------------------------------
// Shared helper — parameterised by `fill` (0 → zeros, !0 → ones)
// ---------------------------------------------------------------------------

/// Counts consecutive bits equal to `fill` from the start of a view.
///
/// `fill` must be `0` (for `leading_zeros`) or `!0` (for `leading_ones`).
#[inline]
fn leading_value_count(words: &[u64], start: usize, bit_len: usize, fill: u64) -> usize {
    let end = start + bit_len;
    let start_offset = start % WORD_BITS;
    let end_rem = end % WORD_BITS;
    let last_wi = (end - 1) / WORD_BITS;

    let mut scanned = 0usize;
    let mut wi = start / WORD_BITS;

    // First word — only bits from start_offset upward are in view.
    let first_val = words[wi] >> start_offset;
    let first_limit = (WORD_BITS - start_offset).min(bit_len);
    let first_count = count_trailing(first_val, fill).min(first_limit);
    if first_count < first_limit {
        return first_count;
    }
    scanned += first_limit;
    wi += 1;

    // Full middle words — SIMD-accelerated value-word scan.
    let mid_end = if end_rem == 0 { last_wi + 1 } else { last_wi };
    if wi < mid_end {
        let value_words = if fill == 0 {
            words[wi..mid_end].leading_zero_words()
        } else {
            words[wi..mid_end].leading_one_words()
        };
        scanned += value_words * WORD_BITS;
        wi += value_words;

        if wi < mid_end {
            return scanned + count_trailing(words[wi], fill).min(WORD_BITS);
        }
    }

    // Last partial word (only when end_rem != 0).
    if end_rem != 0 && wi == last_wi {
        let last_val = words[wi] & low_mask(end_rem);
        scanned += count_trailing(last_val, fill).min(end_rem);
    }

    scanned.min(bit_len)
}

/// Counts trailing bits of a given value within a single u64 word.
///
/// `fill = 0` → `trailing_zeros`, `fill = !0` → `trailing_ones`.
#[inline]
fn count_trailing(val: u64, fill: u64) -> usize {
    if fill == 0 {
        val.trailing_zeros() as usize
    } else {
        (!val).trailing_zeros() as usize
    }
}

impl<'bs> BitStr<'bs> {
    /// Returns the number of consecutive `false` bits from the start of this
    /// view.
    ///
    /// # Complexity
    ///
    /// O(n / 64) worst case, SIMD-accelerated for long runs.
    /// Returns early at the first bit that differs from the expected value.
    ///
    /// # Examples
    ///
    /// ```
    /// use bit_string::BitString;
    ///
    /// let bits = BitString::try_from("00101").unwrap();
    /// let v = bits.as_bit_str();
    /// assert_eq!(v.leading_zeros(), 2);
    /// ```
    #[inline]
    pub fn leading_zeros(&self) -> usize {
        if self.bit_len == 0 {
            return 0;
        }
        leading_value_count(self.source.words(), self.start, self.bit_len, 0)
    }

    /// Returns the number of consecutive `true` bits from the start of this
    /// view.
    ///
    /// # Complexity
    ///
    /// O(n / 64) worst case, SIMD-accelerated for long runs.
    /// Returns early at the first bit that differs from the expected value.
    ///
    /// # Examples
    ///
    /// ```
    /// use bit_string::BitString;
    ///
    /// let bits = BitString::try_from("11010").unwrap();
    /// let v = bits.as_bit_str();
    /// assert_eq!(v.leading_ones(), 2);
    /// ```
    #[inline]
    pub fn leading_ones(&self) -> usize {
        if self.bit_len == 0 {
            return 0;
        }
        leading_value_count(self.source.words(), self.start, self.bit_len, !0)
    }

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
        trailing_value_count(self.source.words(), self.start, self.bit_len, 0)
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
        trailing_value_count(self.source.words(), self.start, self.bit_len, !0)
    }
}

// ---------------------------------------------------------------------------
// Trailing helper — scans from the end backwards
// ---------------------------------------------------------------------------

/// Counts consecutive bits equal to `fill` from the end of a view.
#[inline]
fn trailing_value_count(words: &[u64], start: usize, bit_len: usize, fill: u64) -> usize {
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
        let last_count = count_leading_within(last_val, last_limit, fill);
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
        let trailing_w = if fill == 0 {
            words[mid_first..=wi_end].trailing_zero_words()
        } else {
            words[mid_first..=wi_end].trailing_one_words()
        };
        if trailing_w < nr_words {
            let hit_wi = wi_end - trailing_w;
            scanned += trailing_w * WORD_BITS;
            return scanned + count_leading(words[hit_wi], fill).min(WORD_BITS);
        }
        scanned += trailing_w * WORD_BITS;
    }

    // First word (partial, if start_offset != 0 and not already handled).
    if start_offset > 0 {
        let first_limit = WORD_BITS - start_offset;
        let first_val = words[start_wi] >> start_offset;
        let first_count = count_leading_within(first_val, first_limit, fill);
        scanned += first_count.min(first_limit);
    }

    scanned.min(bit_len)
}

/// Counts leading bits of a given value within a full u64 word.
///
/// `fill = 0` → `leading_zeros`, `fill = !0` → `leading_ones`.
#[inline]
fn count_leading(val: u64, fill: u64) -> usize {
    if fill == 0 {
        val.leading_zeros() as usize
    } else {
        (!val).leading_zeros() as usize
    }
}

/// Counts leading bits of a value within its highest `limit` bits.
///
/// Shifts valid bits to the top of the u64 so that [`u64::leading_zeros`]
/// counts from the highest valid bit downwards.
#[inline]
fn count_leading_within(val: u64, limit: usize, fill: u64) -> usize {
    if limit == 0 {
        return 0;
    }
    let shifted = val << (WORD_BITS - limit);
    if fill == 0 {
        (shifted.leading_zeros() as usize).min(limit)
    } else {
        ((!shifted).leading_zeros() as usize).min(limit)
    }
}

#[cfg(test)]
mod tests_for_leading_zeros;

#[cfg(test)]
mod tests_for_leading_ones;

#[cfg(test)]
mod tests_for_trailing_zeros;

#[cfg(test)]
mod tests_for_trailing_ones;
