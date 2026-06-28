use crate::WORD_BITS;

use super::*;

impl BitString {
    /// Returns `true` if `pattern` matches the bits starting at `index`.
    #[inline]
    pub fn matches_at_str(&self, index: usize, pattern: crate::BitStr<'_>) -> bool {
        self.as_bit_str().matches_at_str(index, pattern)
    }

    // -------------------------------------------------------------------
    // _str methods — argument is BitStr (hs is BitString-aligned)
    // -------------------------------------------------------------------

    /// Returns `true` if `prefix` is a prefix of `self`.
    #[inline]
    pub fn starts_with_str(&self, prefix: crate::BitStr<'_>) -> bool {
        let view = self.as_bit_str();
        if prefix.start % WORD_BITS == 0 {
            view.starts_with_inner::<true, true>(prefix)
        } else {
            view.starts_with_inner::<true, false>(prefix)
        }
    }

    /// Returns `true` if `suffix` is a suffix of `self`.
    #[inline]
    pub fn ends_with_str(&self, suffix: crate::BitStr<'_>) -> bool {
        let view = self.as_bit_str();
        if suffix.bit_len == 0 {
            return true;
        }
        if suffix.bit_len > view.bit_len {
            return false;
        }
        let offset = view.bit_len - suffix.bit_len;
        if suffix.start % WORD_BITS == 0 {
            view.ends_with_inner::<true, true>(suffix, offset)
        } else {
            view.ends_with_inner::<true, false>(suffix, offset)
        }
    }

    // -------------------------------------------------------------------
    // _string methods — both sides are BitString (double word-aligned)
    // -------------------------------------------------------------------

    /// Returns `true` if `prefix` is a prefix of `self`.
    #[inline]
    pub fn starts_with_string(&self, prefix: &BitString) -> bool {
        self.as_bit_str()
            .starts_with_inner::<true, true>(prefix.as_bit_str())
    }

    /// Returns `true` if `suffix` is a suffix of `self`.
    #[inline]
    pub fn ends_with_string(&self, suffix: &BitString) -> bool {
        let view = self.as_bit_str();
        if suffix.bit_len == 0 {
            return true;
        }
        if suffix.bit_len > view.bit_len {
            return false;
        }
        let offset = view.bit_len - suffix.bit_len;
        view.ends_with_inner::<true, true>(suffix.as_bit_str(), offset)
    }
}
