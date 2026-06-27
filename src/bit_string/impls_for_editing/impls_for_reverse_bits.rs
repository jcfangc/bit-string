use alloc::vec::Vec;

use crate::traits::*;
use crate::{WORD_BITS, funcs_for_bits::word_len};

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
        if self.bit_len <= 1 {
            return self.clone();
        }

        let n_words = word_len(self.bit_len);
        let unused = n_words * WORD_BITS - self.bit_len;

        // Step 1: Reverse bits within each word.
        let mut words: Vec<u64> = self.words.iter().map(|w| w.reverse_bits()).collect();
        words.resize(n_words, 0);

        // Step 2: Reverse word order.
        words[..n_words].reverse();

        // Step 3: Right-shift by `unused` to align bits to LSB.
        if unused > 0 {
            words = words.shr(n_words * WORD_BITS, unused);
        }

        BitString::from_words(&words, self.bit_len)
            .expect("reverse_bits: from_words should always succeed for valid inputs")
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
        *self = self.reverse_bits();
    }
}

#[cfg(test)]
mod tests_for_reverse_bits;
