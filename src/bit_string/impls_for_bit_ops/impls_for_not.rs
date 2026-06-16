use super::*;

impl BitString {
    /// Returns `!self` without mutating the input.
    #[inline]
    pub fn not(&self) -> Self {
        Self {
            words: funcs_for_not_core::owned(&self.words, self.bit_len),
            bit_len: self.bit_len,
        }
    }

    /// Replaces `self` with `!self`.
    #[inline]
    pub fn not_assign(&mut self) {
        funcs_for_not_core::assign(&mut self.words, self.bit_len);
    }

    /// Consumes `self`, reuses its backing storage, and returns `!self`.
    #[inline]
    pub fn not_into(mut self) -> Self {
        funcs_for_not_core::assign(&mut self.words, self.bit_len);
        self
    }
}

mod funcs_for_not_core;

#[cfg(test)]
mod tests_for_not;
