use crate::{FILL_ONES, FILL_ZEROS};

use super::BitString;

impl BitString {
    /// Returns the number of consecutive `false` bits from the start.
    ///
    /// Because `BitString` always starts at bit 0, the view is word-aligned.
    #[inline]
    pub fn leading_zeros(&self) -> usize {
        self.as_bit_str()
            .leading_value_bits_inner::<FILL_ZEROS, true>()
    }

    /// Returns the number of consecutive `true` bits from the start.
    #[inline]
    pub fn leading_ones(&self) -> usize {
        self.as_bit_str()
            .leading_value_bits_inner::<FILL_ONES, true>()
    }

    /// Returns the number of consecutive `false` bits from the end.
    #[inline]
    pub fn trailing_zeros(&self) -> usize {
        self.as_bit_str()
            .trailing_value_bits_inner::<FILL_ZEROS, true>()
    }

    /// Returns the number of consecutive `true` bits from the end.
    #[inline]
    pub fn trailing_ones(&self) -> usize {
        self.as_bit_str()
            .trailing_value_bits_inner::<FILL_ONES, true>()
    }
}
