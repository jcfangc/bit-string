use crate::traits::*;

use super::BitString;

impl BitString {
    /// Returns the number of bits set to 1.
    #[inline]
    pub fn count_ones(&self) -> usize {
        self.words.count_ones(self.bit_len)
    }

    /// Returns the number of bits set to 0.
    #[inline]
    pub fn count_zeros(&self) -> usize {
        self.bit_len - self.count_ones()
    }
}

#[cfg(test)]
mod tests_for_count_ones;
