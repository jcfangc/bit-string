use crate::bit_string::errors::BitStringLenMismatch;
use crate::bit_string::traits::*;

use super::BitString;

impl BitString {
    #[inline]
    pub fn or(&self, rhs: &Self) -> Result<Self, BitStringLenMismatch> {
        self.require_same_len(rhs)?;

        Ok(Self {
            words: self.words.or(&rhs.words),
            bit_len: self.bit_len,
        })
    }

    #[inline]
    pub fn or_assign(&mut self, rhs: &Self) -> Result<(), BitStringLenMismatch> {
        self.require_same_len(rhs)?;
        self.words.or_assign(&rhs.words);
        Ok(())
    }
}

#[cfg(test)]
mod tests_for_or;
