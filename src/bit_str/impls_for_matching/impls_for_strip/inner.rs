use crate::BitStr;

impl<'bs> BitStr<'bs> {
    /// `strip_prefix` with compile-time alignment signals.
    #[allow(dead_code)]
    #[inline]
    pub(crate) fn strip_prefix_inner<const HS_WORD_ALIGNED: bool, const ND_WORD_ALIGNED: bool>(
        &self,
        prefix: BitStr<'_>,
    ) -> Option<Self> {
        self.starts_with_inner::<HS_WORD_ALIGNED, ND_WORD_ALIGNED>(prefix)
            .then(|| self.slice_from(prefix.bit_len))
    }

    /// `strip_suffix` with compile-time alignment signals.
    #[allow(dead_code)]
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
