use crate::BitStr;
use crate::WORD_BITS;

mod inner;

impl<'bs> BitStr<'bs> {
    #[inline]
    pub fn matches_at_str(&self, index: usize, pattern: BitStr<'_>) -> bool {
        let hs_aligned = (self.start + index) % WORD_BITS == 0;
        let nd_aligned = pattern.start % WORD_BITS == 0;
        match (hs_aligned, nd_aligned) {
            (true, true) => self.matches_at_inner::<true, true>(index, pattern),
            (true, false) => self.matches_at_inner::<true, false>(index, pattern),
            (false, true) => self.matches_at_inner::<false, true>(index, pattern),
            (false, false) => self.matches_at_inner::<false, false>(index, pattern),
        }
    }
    #[inline]
    pub fn matches_at_string(&self, index: usize, pattern: &crate::BitString) -> bool {
        let p = pattern.as_bit_str();
        if (self.start + index) % WORD_BITS == 0 {
            self.matches_at_inner::<true, true>(index, p)
        } else {
            self.matches_at_inner::<false, true>(index, p)
        }
    }
    #[inline]
    pub fn starts_with_str(&self, prefix: BitStr<'_>) -> bool {
        let hs_aligned = self.start % WORD_BITS == 0;
        let nd_aligned = prefix.start % WORD_BITS == 0;
        match (hs_aligned, nd_aligned) {
            (true, true) => self.starts_with_inner::<true, true>(prefix),
            (true, false) => self.starts_with_inner::<true, false>(prefix),
            (false, true) => self.starts_with_inner::<false, true>(prefix),
            (false, false) => self.starts_with_inner::<false, false>(prefix),
        }
    }
    #[inline]
    pub fn starts_with_string(&self, prefix: &crate::BitString) -> bool {
        let p = prefix.as_bit_str();
        if self.start % WORD_BITS == 0 {
            self.starts_with_inner::<true, true>(p)
        } else {
            self.starts_with_inner::<false, true>(p)
        }
    }
    #[inline]
    pub fn ends_with_str(&self, suffix: BitStr<'_>) -> bool {
        if suffix.bit_len == 0 {
            return true;
        }
        if suffix.bit_len > self.bit_len {
            return false;
        }
        let offset = self.bit_len - suffix.bit_len;
        let hs_aligned = (self.start + offset) % WORD_BITS == 0;
        let nd_aligned = suffix.start % WORD_BITS == 0;
        match (hs_aligned, nd_aligned) {
            (true, true) => self.ends_with_inner::<true, true>(suffix, offset),
            (true, false) => self.ends_with_inner::<true, false>(suffix, offset),
            (false, true) => self.ends_with_inner::<false, true>(suffix, offset),
            (false, false) => self.ends_with_inner::<false, false>(suffix, offset),
        }
    }
    #[inline]
    pub fn ends_with_string(&self, suffix: &crate::BitString) -> bool {
        let s = suffix.as_bit_str();
        if s.bit_len == 0 {
            return true;
        }
        if s.bit_len > self.bit_len {
            return false;
        }
        let offset = self.bit_len - s.bit_len;
        if (self.start + offset) % WORD_BITS == 0 {
            self.ends_with_inner::<true, true>(s, offset)
        } else {
            self.ends_with_inner::<false, true>(s, offset)
        }
    }
}

#[cfg(test)]
mod tests_for_bits_equal_at;
#[cfg(test)]
mod tests_for_ends_with;
#[cfg(test)]
mod tests_for_matches_at;
#[cfg(test)]
mod tests_for_starts_with;
