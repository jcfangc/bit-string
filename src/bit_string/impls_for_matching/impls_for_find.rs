use crate::WORD_BITS;

use super::*;

impl BitString {
    /// Returns `true` if `needle` is contained within `self`.
    ///
    /// `BitString` views are always word-aligned, so the haystack side
    /// passes `HS_WORD_ALIGNED = true`.
    #[inline]
    pub fn contains(&self, needle: crate::BitStr<'_>) -> bool {
        let view = self.as_bit_str();
        let nd_aligned = needle.start % WORD_BITS == 0;
        if nd_aligned {
            view.contains_inner::<true, true>(needle)
        } else {
            view.contains_inner::<true, false>(needle)
        }
    }

    #[inline]
    pub fn find(&self, needle: crate::BitStr<'_>) -> Option<usize> {
        self.as_bit_str().find(needle)
    }

    #[inline]
    pub fn rfind(&self, needle: crate::BitStr<'_>) -> Option<usize> {
        self.as_bit_str().rfind(needle)
    }
}
