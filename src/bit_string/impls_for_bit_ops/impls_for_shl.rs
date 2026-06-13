use super::*;

mod funcs_for_shl_core;

impl BitString {
    /// Returns `self << amount`, filling new bits with zero.
    #[inline]
    pub fn shl(&self, amount: usize) -> Self {
        Self {
            bits: funcs_for_shl_core::owned(&self.bits, self.len, amount),
            len: self.len,
        }
    }

    /// Replaces `self` with `self << amount`, filling new bits with zero.
    #[inline]
    pub fn shl_assign(&mut self, amount: usize) {
        funcs_for_shl_core::assign(&mut self.bits, self.len, amount);
    }

    /// Consumes `self`, reuses its backing storage, and returns `self << amount`.
    #[inline]
    pub fn shl_into(mut self, amount: usize) -> Self {
        funcs_for_shl_core::assign(&mut self.bits, self.len, amount);
        self
    }
}

#[cfg(test)]
mod tests_for_shl;
