use crate::WORD_BITS;

use super::*;

impl BitString {
    /// Returns `true` if `pattern` matches the bits starting at `index`.
    #[inline]
    pub fn matches_at(&self, index: usize, pattern: crate::BitStr<'_>) -> bool {
        self.as_bit_str().matches_at(index, pattern)
    }

    /// Returns `true` if `prefix` is a prefix of `self`.
    ///
    /// `BitString` views are always word-aligned, so the haystack side
    /// passes `HS_WORD_ALIGNED = true`.
    #[inline]
    pub fn starts_with(&self, prefix: crate::BitStr<'_>) -> bool {
        let view = self.as_bit_str();
        let nd_aligned = prefix.start % WORD_BITS == 0;
        if nd_aligned {
            view.starts_with_inner::<true, true>(prefix)
        } else {
            view.starts_with_inner::<true, false>(prefix)
        }
    }

    /// Returns `true` if `suffix` is a suffix of `self`.
    ///
    /// `BitString` views are always word-aligned, so the haystack side
    /// passes `HS_WORD_ALIGNED = true`.
    #[inline]
    pub fn ends_with(&self, suffix: crate::BitStr<'_>) -> bool {
        let view = self.as_bit_str();
        if suffix.bit_len == 0 {
            return true;
        }
        if suffix.bit_len > view.bit_len {
            return false;
        }
        let offset = view.bit_len - suffix.bit_len;
        let nd_aligned = suffix.start % WORD_BITS == 0;
        if nd_aligned {
            view.ends_with_inner::<true, true>(suffix, offset)
        } else {
            view.ends_with_inner::<true, false>(suffix, offset)
        }
    }
}
