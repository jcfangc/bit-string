use crate::traits::*;
use crate::{WORD_BITS, low_mask};

use crate::BitStr;

impl<'bs> BitStr<'bs> {
    /// Returns the number of consecutive `false` bits from the start of this
    /// view.
    ///
    /// # Complexity
    ///
    /// O(n / 64) scalar worst case, SIMD-accelerated for long zero runs.
    /// Returns early at the first non-zero word.
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

        let words = self.source.words();
        let end = self.start + self.bit_len;
        let start_offset = self.start % WORD_BITS;
        let end_rem = end % WORD_BITS;
        let last_wi = (end - 1) / WORD_BITS;

        let mut scanned = 0usize;
        let mut wi = self.start / WORD_BITS;

        // First word — only bits from start_offset upward are in view.
        let first_val = words[wi] >> start_offset;
        let first_limit = (WORD_BITS - start_offset).min(self.bit_len);
        let first_z = (first_val.trailing_zeros() as usize).min(first_limit);
        if first_z < first_limit {
            return first_z;
        }
        scanned += first_limit;
        wi += 1;

        // Full middle words — SIMD-accelerated zero-word scan.
        let mid_end = if end_rem == 0 { last_wi + 1 } else { last_wi };
        if wi < mid_end {
            let zero_words = words[wi..mid_end].leading_zero_words();
            let zero_bits = zero_words * WORD_BITS;
            scanned += zero_bits;
            wi += zero_words;

            // If there's a non-zero word left in the middle range, count its
            // trailing zeros and return early.
            if wi < mid_end {
                let z = (words[wi].trailing_zeros() as usize).min(WORD_BITS);
                return scanned + z;
            }
        }

        // Last partial word (only when end_rem != 0).
        if end_rem != 0 && wi == last_wi {
            let last_val = words[wi] & low_mask(end_rem);
            let last_z = (last_val.trailing_zeros() as usize).min(end_rem);
            scanned += last_z;
        }

        scanned.min(self.bit_len)
    }
}

#[cfg(test)]
mod tests_for_leading_zeros;
