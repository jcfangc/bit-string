use crate::traits::WordsArith;
use crate::{FILL_ONES, FILL_ZEROS, WORD_BITS};

use crate::BitStr;

impl<'bs> BitStr<'bs> {
    /// Counts consecutive trailing bits equal to `FILL`.
    ///
    /// `WORD_ALIGNED`: when `true`, the caller guarantees
    /// `self.start % WORD_BITS == 0`, eliminating the first-word
    /// (from the trailing side) scan at compile time.
    #[inline]
    pub(crate) fn trailing_value_bits_inner<const FILL: u64, const WORD_ALIGNED: bool>(
        &self,
    ) -> usize {
        if self.bit_len == 0 {
            return 0;
        }
        let words = &self.source.words()[self.start / WORD_BITS..];
        let start_offset = (self.start % WORD_BITS) as u32;
        words.trailing_value_bits::<FILL, WORD_ALIGNED>(start_offset, self.bit_len)
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
        if self.start % WORD_BITS == 0 {
            self.trailing_value_bits_inner::<FILL_ZEROS, true>()
        } else {
            self.trailing_value_bits_inner::<FILL_ZEROS, false>()
        }
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
        if self.start % WORD_BITS == 0 {
            self.trailing_value_bits_inner::<FILL_ONES, true>()
        } else {
            self.trailing_value_bits_inner::<FILL_ONES, false>()
        }
    }
}

#[cfg(test)]
mod tests_for_trailing_zeros;

#[cfg(test)]
mod tests_for_trailing_ones;
