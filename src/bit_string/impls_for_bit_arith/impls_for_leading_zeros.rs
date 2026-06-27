use super::BitString;

impl BitString {
    /// Returns the number of consecutive `false` bits from the start.
    ///
    /// Delegates to [`BitStr::leading_zeros`](crate::BitStr::leading_zeros).
    #[inline]
    pub fn leading_zeros(&self) -> usize {
        self.as_bit_str().leading_zeros()
    }

    /// Returns the number of consecutive `true` bits from the start.
    ///
    /// Delegates to [`BitStr::leading_ones`](crate::BitStr::leading_ones).
    #[inline]
    pub fn leading_ones(&self) -> usize {
        self.as_bit_str().leading_ones()
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
