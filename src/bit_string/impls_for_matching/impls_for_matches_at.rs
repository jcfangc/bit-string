use crate::WORD_BITS;

use super::*;

impl BitString {
    /// Returns `true` if `pattern` matches the bits starting at `index`.
    ///
    /// `BitString` is always word-aligned, so `HS_WORD_ALIGNED = true`.
    #[inline]
    pub fn matches_at_str(&self, index: usize, pattern: crate::BitStr<'_>) -> bool {
        let view = self.as_bit_str();
        if index > view.bit_len {
            return false;
        }
        if pattern.bit_len > view.bit_len - index {
            return false;
        }
        if pattern.start % WORD_BITS == 0 {
            view.matches_at_inner::<true, true>(index, pattern)
        } else {
            view.matches_at_inner::<true, false>(index, pattern)
        }
    }

    /// `matches_at_str` when `pattern` is a `BitString` (both aligned).
    #[inline]
    pub fn matches_at_string(&self, index: usize, pattern: &BitString) -> bool {
        let view = self.as_bit_str();
        if index > view.bit_len {
            return false;
        }
        if pattern.bit_len > view.bit_len - index {
            return false;
        }
        view.matches_at_inner::<true, true>(index, pattern.as_bit_str())
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
