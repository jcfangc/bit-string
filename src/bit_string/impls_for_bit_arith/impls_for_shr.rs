use crate::bit_string::traits::*;

use super::BitString;

impl BitString {
    /// Returns `self >> amount`, filling new bits with zero.
    #[inline]
    pub fn shr(&self, amount: usize) -> Self {
        Self {
            words: self.words.shr(self.bit_len, amount),
            bit_len: self.bit_len,
        }
    }

    /// Replaces `self` with `self >> amount`, filling new bits with zero.
    #[inline]
    pub fn shr_assign(&mut self, amount: usize) {
        self.words.shr_assign(self.bit_len, amount);
    }
}

#[cfg(test)]
mod tests_for_shr;
