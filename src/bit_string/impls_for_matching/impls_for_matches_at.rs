use crate::WORD_BITS;

use super::*;

impl BitString {
    /// Returns `true` if `pattern` matches the bits starting at `index`.
    ///
    /// The backing storage is word-aligned, but the comparison window is
    /// aligned only when `index` is word-aligned.
    #[inline]
    pub fn matches_at_str(&self, index: usize, pattern: crate::BitStr<'_>) -> bool {
        self.as_bit_str().matches_at_str(index, pattern)
    }

    /// `matches_at_str` when `pattern` is a `BitString`.
    #[inline]
    pub fn matches_at_string(&self, index: usize, pattern: &BitString) -> bool {
        self.as_bit_str().matches_at_string(index, pattern)
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
        let hs_aligned = offset % WORD_BITS == 0; // self.start == 0, so hs_base == offset
        let nd_aligned = suffix.start % WORD_BITS == 0;
        match (hs_aligned, nd_aligned) {
            (true, true) => view.ends_with_inner::<true, true>(suffix, offset),
            (true, false) => view.ends_with_inner::<true, false>(suffix, offset),
            (false, true) => view.ends_with_inner::<false, true>(suffix, offset),
            (false, false) => view.ends_with_inner::<false, false>(suffix, offset),
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
        let hs_aligned = offset % WORD_BITS == 0;
        if hs_aligned {
            view.ends_with_inner::<true, true>(suffix.as_bit_str(), offset)
        } else {
            view.ends_with_inner::<false, true>(suffix.as_bit_str(), offset)
        }
    }
}
