use crate::BitStr;
use crate::WORD_BITS;

mod inner;

impl<'bs> BitStr<'bs> {
    #[inline]
    pub fn strip_prefix_str(&self, prefix: BitStr<'_>) -> Option<Self> {
        self.starts_with_str(prefix)
            .then(|| self.slice_from(prefix.bit_len))
    }
    #[inline]
    pub fn strip_prefix_string(&self, prefix: &crate::BitString) -> Option<Self> {
        self.starts_with_string(prefix)
            .then(|| self.slice_from(prefix.as_bit_str().bit_len))
    }
    #[inline]
    pub fn strip_suffix_str(&self, suffix: BitStr<'_>) -> Option<Self> {
        self.ends_with_str(suffix)
            .then(|| self.slice_until(self.bit_len - suffix.bit_len))
    }
    #[inline]
    pub fn strip_suffix_string(&self, suffix: &crate::BitString) -> Option<Self> {
        self.ends_with_string(suffix)
            .then(|| self.slice_until(self.bit_len - suffix.as_bit_str().bit_len))
    }
}

#[cfg(test)]
mod tests_for_strip;
