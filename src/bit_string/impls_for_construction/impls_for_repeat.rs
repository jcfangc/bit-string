use super::*;
use crate::bit_string::bits::Bits;
use alloc::vec::Vec;

impl BitString {
    pub fn repeat(value: bool, len: usize) -> Self {
        let word_count = Bits::word_len(len);
        let fill = if value { u64::MAX } else { 0 };
        Self {
            bits: funcs_for_repeat_core::repeat_core(word_count, fill, len),
            len,
        }
    }

    #[inline]
    pub fn zeros(len: usize) -> Self {
        let word_count = Bits::word_len(len);
        // Direct memset — no branch, no SIMD dispatch, no mask needed.
        let mut out = Vec::<u64>::with_capacity(word_count);
        out.resize(word_count, 0);
        Self {
            bits: out.into_boxed_slice(),
            len,
        }
    }

    #[inline]
    pub fn ones(len: usize) -> Self {
        Self::repeat(true, len)
    }
}

mod funcs_for_repeat_core;

#[cfg(test)]
mod tests_for_repeat;
