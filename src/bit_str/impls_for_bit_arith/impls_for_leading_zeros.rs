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
}

#[cfg(test)]
mod tests_for_leading_zeros;

#[cfg(test)]
mod tests_for_leading_ones;
