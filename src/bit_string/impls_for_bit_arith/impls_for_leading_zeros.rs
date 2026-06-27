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
}
