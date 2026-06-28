use crate::BitStr;

impl<'bs> BitStr<'bs> {
    /// Strips `prefix` from the start, returning the remaining sub-view.
    #[inline]
    pub fn strip_prefix(&self, prefix: BitStr<'_>) -> Option<Self> {
        self.starts_with_str(prefix)
            .then(|| self.slice_from(prefix.bit_len))
    }

    /// Strips `suffix` from the end, returning the remaining sub-view.
    #[inline]
    pub fn strip_suffix(&self, suffix: BitStr<'_>) -> Option<Self> {
        self.ends_with_str(suffix)
            .then(|| self.slice_until(self.bit_len - suffix.bit_len))
    }
}

#[cfg(test)]
mod tests_for_strip;
