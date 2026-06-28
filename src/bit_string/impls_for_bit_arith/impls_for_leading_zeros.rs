use super::BitString;
use crate::bit_str::impls_for_bit_arith::impls_for_leading_zeros::leading_value_count;
use crate::{FILL_ONES, FILL_ZEROS};

impl BitString {
    /// Returns the number of consecutive `false` bits from the start.
    ///
    /// Because `BitString` views always start at bit 0, the first word is
    /// word-aligned — we pass `ALIGNED = true` so the compiler eliminates
    /// the first-word TZCNT.
    #[inline]
    pub fn leading_zeros(&self) -> usize {
        if self.bit_len == 0 {
            return 0;
        }
        leading_value_count::<FILL_ZEROS, true>(self.words(), 0, self.bit_len)
    }

    /// Returns the number of consecutive `true` bits from the start.
    #[inline]
    pub fn leading_ones(&self) -> usize {
        if self.bit_len == 0 {
            return 0;
        }
        leading_value_count::<FILL_ONES, true>(self.words(), 0, self.bit_len)
    }

    /// Returns the number of consecutive `false` bits from the end.
    ///
    /// Delegates to [`BitStr::trailing_zeros`](crate::BitStr::trailing_zeros).
    #[inline]
    pub fn trailing_zeros(&self) -> usize {
        self.as_bit_str().trailing_zeros()
    }

    /// Returns the number of consecutive `true` bits from the end.
    ///
    /// Delegates to [`BitStr::trailing_ones`](crate::BitStr::trailing_ones).
    #[inline]
    pub fn trailing_ones(&self) -> usize {
        self.as_bit_str().trailing_ones()
    }
}
