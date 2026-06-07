use crate::bit_string::errors::ParseBitStringError;

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
        Self::from_bool_iter(values)
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
