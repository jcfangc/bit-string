use crate::BitStr;
use crate::WORD_BITS;

impl<'bs> BitStr<'bs> {
    /// Strips `prefix` from the start, returning the remaining sub-view.
    #[inline]
    pub fn strip_prefix_str(&self, prefix: BitStr<'_>) -> Option<Self> {
        self.starts_with_str(prefix)
            .then(|| self.slice_from(prefix.bit_len))
    }

    /// `strip_prefix_str` when `prefix` is a [`BitString`](crate::BitString).
    #[inline]
    pub fn strip_prefix_string(&self, prefix: &crate::BitString) -> Option<Self> {
        self.starts_with_string(prefix)
            .then(|| self.slice_from(prefix.as_bit_str().bit_len))
    }

    /// `strip_prefix` with compile-time alignment signals.
    #[inline]
    pub(crate) fn strip_prefix_inner<const HS_WORD_ALIGNED: bool, const ND_WORD_ALIGNED: bool>(
        &self,
        prefix: BitStr<'_>,
    ) -> Option<Self> {
        self.starts_with_inner::<HS_WORD_ALIGNED, ND_WORD_ALIGNED>(prefix)
            .then(|| self.slice_from(prefix.bit_len))
    }

    /// Strips `suffix` from the end, returning the remaining sub-view.
    #[inline]
    pub fn strip_suffix_str(&self, suffix: BitStr<'_>) -> Option<Self> {
        self.ends_with_str(suffix)
            .then(|| self.slice_until(self.bit_len - suffix.bit_len))
    }

    /// `strip_suffix_str` when `suffix` is a [`BitString`](crate::BitString).
    #[inline]
    pub fn strip_suffix_string(&self, suffix: &crate::BitString) -> Option<Self> {
        self.ends_with_string(suffix)
            .then(|| self.slice_until(self.bit_len - suffix.as_bit_str().bit_len))
    }

    /// `strip_suffix` with compile-time alignment signals.
    #[inline]
    pub(crate) fn strip_suffix_inner<const HS_WORD_ALIGNED: bool, const ND_WORD_ALIGNED: bool>(
        &self,
        suffix: BitStr<'_>,
    ) -> Option<Self> {
        let offset = self.bit_len - suffix.bit_len;
        self.ends_with_inner::<HS_WORD_ALIGNED, ND_WORD_ALIGNED>(suffix, offset)
            .then(|| self.slice_until(self.bit_len - suffix.bit_len))
    }
}

#[cfg(test)]
mod tests_for_strip;
