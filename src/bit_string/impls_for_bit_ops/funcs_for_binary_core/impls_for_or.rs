use super::*;

use funcs_for_binary_core::{OP_OR, assign, owned};

impl BitString {
    /// Returns `self & rhs` without mutating either input.
    #[inline]
    pub fn or(&self, rhs: &Self) -> Result<Self, BitStringLenMismatch> {
        self.require_same_len(rhs)?;

        Ok(Self {
            words: owned::<OP_OR>(&self.words, &rhs.words),
            bit_len: self.bit_len,
        })
    }

    /// Replaces `self` with `self & rhs`.
    #[inline]
    pub fn or_assign(&mut self, rhs: &Self) -> Result<(), BitStringLenMismatch> {
        self.require_same_len(rhs)?;
        assign::<OP_OR>(&mut self.words, &rhs.words);
        Ok(())
    }

    /// Consumes `self`, reuses its backing storage, and returns `self & rhs`.
    #[inline]
    pub fn or_into(mut self, rhs: &Self) -> Result<Self, BitStringLenMismatch> {
        self.require_same_len(rhs)?;
        assign::<OP_OR>(&mut self.words, &rhs.words);
        Ok(self)
    }
}

#[cfg(test)]
mod tests_for_or;
