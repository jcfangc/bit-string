use super::*;
use crate::bit_string::traits::*;
use crate::funcs_for_bits::*;
use alloc::vec::Vec;

impl BitString {
    pub fn repeat(value: bool, bit_len: usize) -> Self {
        let word_count = word_len(bit_len);
        let fill = if value { u64::MAX } else { 0 };
        let mut out = Vec::<u64>::with_capacity(word_count);
        out.resize(word_count, fill);
        out.mask_unused_bits(bit_len);
        Self {
            words: out,
            bit_len,
        }
    }

    #[inline]
    pub fn zeros(bit_len: usize) -> Self {
        let word_count = word_len(bit_len);
        // Direct memset — no branch, no mask needed.
        let mut out = Vec::<u64>::with_capacity(word_count);
        out.resize(word_count, 0);
        Self {
            words: out,
            bit_len,
        }
    }

    #[inline]
    pub fn ones(bit_len: usize) -> Self {
        let word_count = word_len(bit_len);
        let mut out = Vec::<u64>::with_capacity(word_count);
        out.resize(word_count, u64::MAX);
        out.mask_unused_bits(bit_len);
        Self {
            words: out,
            bit_len,
        }
    }
}

#[cfg(test)]
mod tests_for_repeat;
