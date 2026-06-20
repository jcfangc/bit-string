use crate::bit_string::traits::*;

use super::BitString;

impl BitString {
    /// Returns `!self` without mutating the input.
    #[inline]
    pub fn not(&self) -> Self {
        Self {
            words: self.words.not(self.bit_len),
            bit_len: self.bit_len,
        }
    }

    /// Replaces `self` with `!self`.
    #[inline]
    pub fn not_assign(&mut self) {
        self.words.not_assign(self.bit_len);
    }

    /// Consumes `self`, reuses its backing storage, and returns `!self`.
    #[inline]
    pub fn not_into(mut self) -> Self {
        self.words.not_assign(self.bit_len);
        self
    }
}

#[cfg(test)]
mod tests_for_not;
