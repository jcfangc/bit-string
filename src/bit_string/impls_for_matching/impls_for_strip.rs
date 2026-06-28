use crate::WORD_BITS;

use super::*;

impl BitString {
    /// Strips `prefix` from the start, returning the remaining `BitString`.
    #[inline]
    pub fn strip_prefix_str(&self, prefix: crate::BitStr<'_>) -> Option<Self> {
        self.as_bit_str()
            .starts_with_str(prefix)
            .then(|| self.slice_from(prefix.bit_len))
    }

    /// `strip_prefix_str` when both sides are `BitString`.
    #[inline]
    pub fn strip_prefix_string(&self, prefix: &BitString) -> Option<Self> {
        self.as_bit_str()
            .starts_with_string(prefix)
            .then(|| self.slice_from(prefix.as_bit_str().bit_len))
    }

    /// Strips `suffix` from the end, returning the remaining `BitString`.
    #[inline]
    pub fn strip_suffix_str(&self, suffix: crate::BitStr<'_>) -> Option<Self> {
        self.as_bit_str()
            .ends_with_str(suffix)
            .then(|| self.slice_until(self.bit_len - suffix.bit_len))
    }

    /// `strip_suffix_str` when both sides are `BitString`.
    #[inline]
    pub fn strip_suffix_string(&self, suffix: &BitString) -> Option<Self> {
        self.as_bit_str()
            .ends_with_string(suffix)
            .then(|| self.slice_until(self.bit_len - suffix.as_bit_str().bit_len))
    }
}
