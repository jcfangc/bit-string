use super::*;

impl BitString {
    #[inline]
    pub fn strip_prefix(&self, prefix: crate::BitStr<'_>) -> Option<Self> {
        self.as_bit_str()
            .starts_with(prefix)
            .then(|| self.slice_from(prefix.bit_len))
    }

    #[inline]
    pub fn strip_suffix(&self, suffix: crate::BitStr<'_>) -> Option<Self> {
        self.as_bit_str()
            .ends_with(suffix)
            .then(|| self.slice_until(self.bit_len - suffix.bit_len))
    }
}
