use super::*;

impl BitString {
    /// Returns `self & rhs` without mutating either input.
    #[inline]
    pub fn and(&self, rhs: &Self) -> Result<Self, BitStringLenMismatch> {
        self.require_same_len(rhs)?;

        Ok(Self {
            bits: funcs_for_core::owned(&self.bits, &rhs.bits),
            len: self.len,
        })
    }

    /// Replaces `self` with `self & rhs`.
    #[inline]
    pub fn and_assign(&mut self, rhs: &Self) -> Result<(), BitStringLenMismatch> {
        self.require_same_len(rhs)?;
        funcs_for_core::assign(&mut self.bits, &rhs.bits);
        Ok(())
    }

    /// Consumes `self`, reuses its backing storage, and returns `self & rhs`.
    #[inline]
    pub fn and_into(mut self, rhs: &Self) -> Result<Self, BitStringLenMismatch> {
        self.require_same_len(rhs)?;
        funcs_for_core::assign(&mut self.bits, &rhs.bits);
        Ok(self)
    }
}

mod funcs_for_core;

#[cfg(test)]
mod tests_for_and;
