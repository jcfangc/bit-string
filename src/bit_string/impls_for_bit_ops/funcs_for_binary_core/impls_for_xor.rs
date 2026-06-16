use super::*;

use funcs_for_binary_core::{OP_XOR, assign, owned};

impl BitString {
    /// Returns `self & rhs` without mutating either input.
    #[inline]
    pub fn xor(&self, rhs: &Self) -> Result<Self, BitStringLenMismatch> {
        self.require_same_len(rhs)?;

        Ok(Self {
            words: owned::<OP_XOR>(&self.words, &rhs.words),
            bit_len: self.bit_len,
        })
    }

    /// Replaces `self` with `self & rhs`.
    #[inline]
    pub fn xor_assign(&mut self, rhs: &Self) -> Result<(), BitStringLenMismatch> {
        self.require_same_len(rhs)?;
        assign::<OP_XOR>(&mut self.words, &rhs.words);
        Ok(())
    }

    /// Consumes `self`, reuses its backing stxorage, and returns `self & rhs`.
    #[inline]
    pub fn xor_into(mut self, rhs: &Self) -> Result<Self, BitStringLenMismatch> {
        self.require_same_len(rhs)?;
        assign::<OP_XOR>(&mut self.words, &rhs.words);
        Ok(self)
    }
}

#[cfg(test)]
mod tests_for_xor;
