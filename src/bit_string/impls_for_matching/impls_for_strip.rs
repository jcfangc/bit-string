use crate::WORD_BITS;

use super::*;

impl BitString {
    /// Strips `prefix` from the start, returning the remaining `BitString`.
    #[inline]
    pub fn strip_prefix_str(&self, prefix: crate::BitStr<'_>) -> Option<Self> {
        let view = self.as_bit_str();
        let ok = if prefix.start % WORD_BITS == 0 {
            view.starts_with_inner::<true, true>(prefix)
        } else {
            view.starts_with_inner::<true, false>(prefix)
        };
        ok.then(|| self.slice_from(prefix.bit_len))
    }

    /// `strip_prefix_str` when both sides are `BitString`.
    #[inline]
    pub fn strip_prefix_string(&self, prefix: &BitString) -> Option<Self> {
        let ok = self
            .as_bit_str()
            .starts_with_inner::<true, true>(prefix.as_bit_str());
        ok.then(|| self.slice_from(prefix.as_bit_str().bit_len))
    }

    /// Strips `suffix` from the end, returning the remaining `BitString`.
    #[inline]
    pub fn strip_suffix_str(&self, suffix: crate::BitStr<'_>) -> Option<Self> {
        self.ends_with_str(suffix)
            .then(|| self.slice_until(self.bit_len - suffix.bit_len))
    }

    /// `strip_suffix_str` when both sides are `BitString`.
    #[inline]
    pub fn strip_suffix_string(&self, suffix: &BitString) -> Option<Self> {
        self.ends_with_string(suffix)
            .then(|| self.slice_until(self.bit_len - suffix.bit_len))
    }
}
