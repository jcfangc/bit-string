use super::*;

impl BitString {
    /// Returns `self & rhs` without mutating either input.
    #[inline]
    pub fn and(&self, rhs: &Self) -> Result<Self, BitStringLenMismatch> {
        self.require_same_len(rhs)?;

        Ok(Self {
            bits: owned::<OP_AND>(&self.bits, &rhs.bits),
            len: self.len,
        })
    }

    /// Replaces `self` with `self & rhs`.
    #[inline]
    pub fn and_assign(&mut self, rhs: &Self) -> Result<(), BitStringLenMismatch> {
        self.require_same_len(rhs)?;
        assign::<OP_AND>(&mut self.bits, &rhs.bits);
        Ok(())
    }

    /// Consumes `self`, reuses its backing storage, and returns `self & rhs`.
    #[inline]
    pub fn and_into(mut self, rhs: &Self) -> Result<Self, BitStringLenMismatch> {
        self.require_same_len(rhs)?;
        assign::<OP_AND>(&mut self.bits, &rhs.bits);
        Ok(self)
    }
}

#[cfg(test)]
mod tests_for_and;
