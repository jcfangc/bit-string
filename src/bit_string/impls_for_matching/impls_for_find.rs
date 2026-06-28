use crate::WORD_BITS;

use super::*;

impl BitString {
    // -------------------------------------------------------------------
    // _str methods — needle is BitStr (hs is BitString-aligned)
    // -------------------------------------------------------------------

    /// Returns `true` if `needle` is contained within `self`.
    #[inline]
    pub fn contains_str(&self, needle: crate::BitStr<'_>) -> bool {
        let view = self.as_bit_str();
        if needle.start % WORD_BITS == 0 {
            view.contains_inner::<true, true>(needle)
        } else {
            view.contains_inner::<true, false>(needle)
        }
    }

    #[inline]
    pub fn find_str(&self, needle: crate::BitStr<'_>) -> Option<usize> {
        self.as_bit_str().find_inner::<true>(needle)
    }

    #[inline]
    pub fn rfind_str(&self, needle: crate::BitStr<'_>) -> Option<usize> {
        self.as_bit_str().rfind_inner::<true>(needle)
    }

    // -------------------------------------------------------------------
    // _string methods — both sides are BitString (double word-aligned)
    // -------------------------------------------------------------------

    /// Returns `true` if `needle` is contained within `self`.
    #[inline]
    pub fn contains_string(&self, needle: &BitString) -> bool {
        self.as_bit_str()
            .contains_inner::<true, true>(needle.as_bit_str())
    }

    #[inline]
    pub fn find_string(&self, needle: &BitString) -> Option<usize> {
        self.as_bit_str().find_inner::<true>(needle.as_bit_str())
    }

    #[inline]
    pub fn rfind_string(&self, needle: &BitString) -> Option<usize> {
        self.as_bit_str().rfind_inner::<true>(needle.as_bit_str())
    }
}
