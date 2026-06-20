use crate::bit_string::errors::BitStringLenMismatch;
use crate::bit_string::traits::*;

use super::BitString;

impl BitString {
    /// Returns `self & rhs` without mutating either input.
    #[inline]
    pub fn and(&self, rhs: &Self) -> Result<Self, BitStringLenMismatch> {
        self.require_same_len(rhs)?;

        Ok(Self {
            words: self.words.and(&rhs.words),
            bit_len: self.bit_len,
        })
    }

    /// Replaces `self` with `self & rhs`.
    #[inline]
    pub fn and_assign(&mut self, rhs: &Self) -> Result<(), BitStringLenMismatch> {
        self.require_same_len(rhs)?;
        self.words.and_assign(&rhs.words);
        Ok(())
    }

    /// Consumes `self`, reuses its backing storage, and returns `self & rhs`.
    #[inline]
    pub fn and_into(mut self, rhs: &Self) -> Result<Self, BitStringLenMismatch> {
        self.require_same_len(rhs)?;
        self.words.and_assign(&rhs.words);
        Ok(self)
    }
}

#[cfg(test)]
mod tests_for_and;
