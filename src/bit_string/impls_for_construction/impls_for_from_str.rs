use super::*;
use crate::bit_string::errors::ParseBitStringError;
use core::str::FromStr;

impl TryFrom<&str> for BitString {
    type Error = ParseBitStringError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let src = value.as_ptr();
        let len = value.len();

        match funcs_for_pack_str_core::str_core(src, len) {
            Ok(bits) => Ok(Self { bits, len }),
            Err((index, byte)) => Err(ParseBitStringError { index, byte }),
        }
    }
}

impl FromStr for BitString {
    type Err = ParseBitStringError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

mod funcs_for_pack_str_core;
