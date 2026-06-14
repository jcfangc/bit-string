use super::*;
use crate::bit_string::bits::Bits;

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
        Self::repeat(false, len)
    }

    #[inline]
    pub fn ones(len: usize) -> Self {
        Self::repeat(true, len)
    }
}

mod funcs_for_repeat_core;

#[cfg(test)]
mod tests_for_repeat;
