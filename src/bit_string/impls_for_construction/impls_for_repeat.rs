use super::*;
use crate::bit_string::bits::Bits;
use alloc::vec::Vec;

impl BitString {
    pub fn repeat(value: bool, len: usize) -> Self {
        let word_count = Bits::word_len(len);
        let fill = if value { u64::MAX } else { 0 };
        let mut out = Vec::<u64>::with_capacity(word_count);
        out.resize(word_count, fill);
        Bits::mask_unused(&mut out, len);
        Self { bits: out, len }
    }

    #[inline]
    pub fn zeros(len: usize) -> Self {
        let word_count = Bits::word_len(len);
        // Direct memset — no branch, no mask needed.
        let mut out = Vec::<u64>::with_capacity(word_count);
        out.resize(word_count, 0);
        Self { bits: out, len }
    }

    #[inline]
    pub fn ones(len: usize) -> Self {
        let word_count = Bits::word_len(len);
        let mut out = Vec::<u64>::with_capacity(word_count);
        out.resize(word_count, u64::MAX);
        Bits::mask_unused(&mut out, len);
        Self { bits: out, len }
    }
}

#[cfg(test)]
mod tests_for_repeat;
