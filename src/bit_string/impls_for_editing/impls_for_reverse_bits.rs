use crate::WORD_BITS;

use super::BitString;

impl BitString {
    /// Returns a new `BitString` with the bits in reverse order.
    ///
    /// Bit at position `i` moves to position `bit_len - 1 - i`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bit_string::BitString;
    ///
    /// let bits = BitString::try_from("10011").unwrap();
    /// let rev = bits.reverse_bits();
    /// assert_eq!(rev, BitString::try_from("11001").unwrap());
    /// ```
    #[inline]
    pub fn reverse_bits(&self) -> Self {
        let mut reversed = self.clone();
        reversed.reverse_bits_assign();
        reversed
    }

    /// Reverses the bit order in place.
    ///
    /// Bit at position `i` moves to position `bit_len - 1 - i`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bit_string::BitString;
    ///
    /// let mut bits = BitString::try_from("10011").unwrap();
    /// bits.reverse_bits_assign();
    /// assert_eq!(bits, BitString::try_from("11001").unwrap());
    /// ```
    #[inline]
    pub fn reverse_bits_assign(&mut self) {
        if self.bit_len <= 1 {
            return;
        }

        self.words.reverse();
        for word in &mut self.words {
            *word = word.reverse_bits();
        }

        let unused = self.words.len() * WORD_BITS - self.bit_len;
        if unused == 0 {
            return;
        }

        let last_word_index = self.words.len() - 1;
        for index in 0..last_word_index {
            self.words[index] =
                (self.words[index] >> unused) | (self.words[index + 1] << (WORD_BITS - unused));
        }
        self.words[last_word_index] >>= unused;
    }
}

#[cfg(test)]
mod tests_for_reverse_bits;
