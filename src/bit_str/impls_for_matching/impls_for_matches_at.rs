use crate::BitString;

use crate::BitStr;

impl<'bs> BitStr<'bs> {
    /// Compare `needle` bits against `self` starting at `offset`.
    ///
    /// Delegates to [`BitString::bits_equal_at`] with the view's start
    /// offset added.
    #[inline]
    pub(crate) fn bits_equal_at(&self, offset: usize, needle: &BitString) -> bool {
        self.source.bits_equal_at(self.start + offset, needle)
    }

    /// Returns `true` if `pattern` matches the bits starting at `index`.
    #[inline]
    pub fn matches_at(&self, index: usize, pattern: &BitString) -> bool {
        if index > self.bit_len {
            return false;
        }
        if pattern.bit_len() > self.bit_len - index {
            return false;
        }
        self.bits_equal_at(index, pattern)
    }

    /// Returns `true` if `prefix` is a prefix of `self`.
    #[inline]
    pub fn starts_with(&self, prefix: &BitString) -> bool {
        self.matches_at(0, prefix)
    }

    /// Returns `true` if `suffix` is a suffix of `self`.
    #[inline]
    pub fn ends_with(&self, suffix: &BitString) -> bool {
        if suffix.bit_len() == 0 {
            return true;
        }
        if suffix.bit_len() > self.bit_len {
            return false;
        }
        self.bits_equal_at(self.bit_len - suffix.bit_len(), suffix)
    }
}

#[cfg(test)]
mod tests_for_matches_at;
