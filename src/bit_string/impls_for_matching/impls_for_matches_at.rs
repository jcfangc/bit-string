use super::*;

impl BitString {
    /// Returns `true` if `pattern` matches the bits starting at `index`.
    #[inline]
    pub fn matches_at(&self, index: usize, pattern: crate::BitStr<'_>) -> bool {
        self.as_bit_str().matches_at(index, pattern)
    }

    /// Returns `true` if `prefix` is a prefix of `self`.
    #[inline]
    pub fn starts_with(&self, prefix: crate::BitStr<'_>) -> bool {
        self.as_bit_str().starts_with(prefix)
    }

    /// Returns `true` if `suffix` is a suffix of `self`.
    #[inline]
    pub fn ends_with(&self, suffix: crate::BitStr<'_>) -> bool {
        self.as_bit_str().ends_with(suffix)
    }
}
