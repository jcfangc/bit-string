use crate::traits::BitsArith;
use crate::{FILL_ONES, FILL_ZEROS, WORD_BITS};

use crate::BitStr;

impl<'bs> BitStr<'bs> {
    /// Counts consecutive leading bits equal to `FILL`.
    ///
    /// `WORD_ALIGNED`: when `true`, the caller guarantees
    /// `self.start % WORD_BITS == 0`, eliminating the first-word TZCNT at
    /// compile time.
    #[inline]
    pub(crate) fn leading_value_bits_inner<const FILL: u64, const WORD_ALIGNED: bool>(
        &self,
    ) -> usize {
        if self.bit_len == 0 {
            return 0;
        }
        let words = &self.source.words()[self.start / WORD_BITS..];
        let start_offset = (self.start % WORD_BITS) as u32;
        words.leading_value_bits::<FILL, WORD_ALIGNED>(start_offset, self.bit_len)
    }

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
        if self.start % WORD_BITS == 0 {
            self.leading_value_bits_inner::<FILL_ZEROS, true>()
        } else {
            self.leading_value_bits_inner::<FILL_ZEROS, false>()
        }
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
        if self.start % WORD_BITS == 0 {
            self.leading_value_bits_inner::<FILL_ONES, true>()
        } else {
            self.leading_value_bits_inner::<FILL_ONES, false>()
        }
    }
}

#[cfg(test)]
mod tests_for_leading_zeros;

#[cfg(test)]
mod tests_for_leading_ones;
