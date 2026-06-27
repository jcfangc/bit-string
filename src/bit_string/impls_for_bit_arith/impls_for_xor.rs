use crate::bit_string::errors::BitStringLenMismatch;
use crate::traits::*;

use super::BitString;

impl BitString {
    #[inline]
    pub fn xor(&self, rhs: &Self) -> Result<Self, BitStringLenMismatch> {
        self.require_same_len(rhs)?;

        Ok(Self {
            words: self.words.xor(&rhs.words),
            bit_len: self.bit_len,
        })
    }

    #[inline]
    pub fn xor_assign(&mut self, rhs: &Self) -> Result<(), BitStringLenMismatch> {
        self.require_same_len(rhs)?;
        self.words.xor_assign(&rhs.words);
        Ok(())
    }
}

#[cfg(test)]
mod tests_for_xor;
