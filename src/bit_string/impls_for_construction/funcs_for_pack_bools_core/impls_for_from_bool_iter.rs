use crate::BitString;
use alloc::vec::Vec;

impl BitString {
    pub(crate) fn from_bool_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = bool>,
    {
        let bytes: Vec<u8> = iter.into_iter().map(|v| v as u8).collect();
        let bit_len = bytes.len();
        let src = bytes.as_ptr();
        Self {
            words: super::bools_core(src, bit_len),
            bit_len,
        }
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

#[cfg(test)]
mod tests_for_from_bool_iter;
