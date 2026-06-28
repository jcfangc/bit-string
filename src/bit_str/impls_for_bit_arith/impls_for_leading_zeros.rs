use crate::traits::*;
use crate::{FILL_ONES, FILL_ZEROS, WORD_BITS, low_mask};

use crate::BitStr;

// ---------------------------------------------------------------------------
// Shared helper — parameterised by `FILL` (0 → zeros, !0 → ones)
// ---------------------------------------------------------------------------

/// Counts consecutive bits equal to `FILL` from the start of a view.
#[inline]
fn leading_value_count<const FILL: u64>(words: &[u64], start: usize, bit_len: usize) -> usize {
    let end = start + bit_len;
    let start_offset = start % WORD_BITS;
    let end_rem = end % WORD_BITS;
    let last_wi = (end - 1) / WORD_BITS;

    let mut scanned = 0usize;
    let mut wi = start / WORD_BITS;

    // First word — only bits from start_offset upward are in view.
    let first_val = words[wi] >> start_offset;
    let first_limit = (WORD_BITS - start_offset).min(bit_len);
    let first_count = count_trailing::<FILL>(first_val).min(first_limit);
    if first_count < first_limit {
        return first_count;
    }
    scanned += first_limit;
    wi += 1;

    // Full middle words — SIMD-accelerated value-word scan.
    let mid_end = if end_rem == 0 { last_wi + 1 } else { last_wi };
    if wi < mid_end {
        let value_words = words[wi..mid_end].leading_value_words::<FILL>();
        scanned += value_words * WORD_BITS;
        wi += value_words;

        if wi < mid_end {
            return scanned + count_trailing::<FILL>(words[wi]).min(WORD_BITS);
        }
    }

    // Last partial word (only when end_rem != 0).
    if end_rem != 0 && wi == last_wi {
        let last_val = words[wi] & low_mask(end_rem);
        scanned += count_trailing::<FILL>(last_val).min(end_rem);
    }

    scanned.min(bit_len)
}

/// Counts trailing bits of a given value within a single u64 word.
#[inline]
fn count_trailing<const FILL: u64>(val: u64) -> usize {
    if FILL == 0 {
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
        leading_value_count::<FILL_ZEROS>(self.source.words(), self.start, self.bit_len)
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
        leading_value_count::<FILL_ONES>(self.source.words(), self.start, self.bit_len)
    }
}

#[cfg(test)]
mod tests_for_leading_zeros;

#[cfg(test)]
mod tests_for_leading_ones;
