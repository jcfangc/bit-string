use crate::bit_string::traits::*;

use super::BitString;

impl BitString {
    /// Returns `self << amount`, filling new bits with zero.
    #[inline]
    pub fn shl(&self, amount: usize) -> Self {
        Self {
            words: self.words.shl(self.bit_len, amount),
            bit_len: self.bit_len,
        }
    }

    /// Replaces `self` with `self << amount`, filling new bits with zero.
    #[inline]
    pub fn shl_assign(&mut self, amount: usize) {
        self.words.shl_assign(self.bit_len, amount);
    }
}

#[cfg(test)]
mod tests_for_shl;
