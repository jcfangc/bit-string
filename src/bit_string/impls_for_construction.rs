use crate::bit_string::{bits::Bits, errors::ParseBitStringError};

use super::*;

use core::str::FromStr;

use alloc::vec::Vec;

impl BitString {
    #[inline]
    pub fn new() -> Self {
        Self {
            bits: Vec::new().into_boxed_slice(),
            len: 0,
        }
    }

    pub fn repeat(value: bool, len: usize) -> Self {
        let word_count = Bits::word_len(len);
        let fill = if value { u64::MAX } else { 0 };
        Self {
            bits: funcs_for_repeat_core::owned(word_count, fill, len),
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

    pub(crate) fn from_bool_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = bool>,
    {
        let mut bits = Vec::<u64>::new();
        let mut len = 0usize;

        for value in iter {
            if len % WORD_BITS == 0 {
                bits.push(0);
            }

            if value {
                let word = len / WORD_BITS;
                let offset = len % WORD_BITS;
                bits[word] |= 1u64 << offset;
            }

            len += 1;
        }

        Self {
            bits: bits.into_boxed_slice(),
            len,
        }
    }

    /// Constructs a bit string from packed little-endian words.
    ///
    /// The input must contain exactly enough words for `len`.
    /// Unused high bits in the last word are masked out.
    pub fn from_words(words: &[u64], len: usize) -> Option<Self> {
        let word_count = Bits::word_len(len);

        if words.len() != word_count {
            return None;
        }

        let mut bits = Vec::with_capacity(word_count);
        bits.extend_from_slice(words);

        let mut bits = bits.into_boxed_slice();
        Bits::mask_unused(&mut bits, len);

        Some(Self { bits, len })
    }
}

impl Default for BitString {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl FromIterator<bool> for BitString {
    #[inline]
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = bool>,
    {
        Self::from_bool_iter(iter)
    }
}

impl From<&[bool]> for BitString {
    #[inline]
    fn from(values: &[bool]) -> Self {
        Self::from_bool_iter(values.iter().copied())
    }
}

impl<const N: usize> From<[bool; N]> for BitString {
    #[inline]
    fn from(values: [bool; N]) -> Self {
        // SAFETY:
        // - `bool` has layout/size/alignment 1, so `*const bool` → `*const u8`
        //   is a valid pointer cast.
        // - Valid bool values are 0x00 (false) or 0x01 (true).
        let src = values.as_ptr() as *const u8;
        Self {
            bits: funcs_for_pack_bools_core::owned(src, N),
            len: N,
        }
    }
}

impl TryFrom<&str> for BitString {
    type Error = ParseBitStringError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let iter = value.bytes().enumerate().map(|(index, byte)| match byte {
            b'0' => Ok(false),
            b'1' => Ok(true),
            byte => Err(ParseBitStringError { index, byte }),
        });

        let mut bools = Vec::with_capacity(value.len());

        for item in iter {
            bools.push(item?);
        }

        Ok(Self::from_bool_iter(bools))
    }
}

impl FromStr for BitString {
    type Err = ParseBitStringError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

mod funcs_for_repeat_core;

mod funcs_for_pack_bools_core;

#[cfg(test)]
mod tests_for_repeat;

#[cfg(test)]
mod tests_for_from_bool_iter;

#[cfg(test)]
mod tests_for_from_words;
