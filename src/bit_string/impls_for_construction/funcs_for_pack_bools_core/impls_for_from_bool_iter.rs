use crate::BitString;
use alloc::vec::Vec;

impl BitString {
    pub(crate) fn from_bool_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = bool>,
    {
        let bytes: Vec<u8> = iter.into_iter().map(|v| v as u8).collect();
        let len = bytes.len();
        let src = bytes.as_ptr();
        Self {
            bits: super::bools_core(src, len),
            len,
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
