use crate::BitStr;
use crate::WORD_BITS;

mod inner;

impl<'bs> BitStr<'bs> {
    #[inline]
    pub fn contains_str(&self, needle: BitStr<'_>) -> bool {
        let hs_aligned = self.start % WORD_BITS == 0;
        let nd_aligned = needle.start % WORD_BITS == 0;
        match (hs_aligned, nd_aligned) {
            (true, true) => self.contains_inner::<true, true>(needle),
            (true, false) => self.contains_inner::<true, false>(needle),
            (false, true) => self.contains_inner::<false, true>(needle),
            (false, false) => self.contains_inner::<false, false>(needle),
        }
    }
    #[inline]
    pub fn contains_string(&self, needle: &crate::BitString) -> bool {
        let n = needle.as_bit_str();
        if self.start % WORD_BITS == 0 {
            self.contains_inner::<true, true>(n)
        } else {
            self.contains_inner::<false, true>(n)
        }
    }
    #[inline]
    pub fn find_str(&self, needle: BitStr<'_>) -> Option<usize> {
        let hs_a = self.start % WORD_BITS == 0;
        let nd_a = needle.start % WORD_BITS == 0;
        match (hs_a, nd_a) {
            (true, true) => self.find_inner::<true, true>(needle),
            (true, false) => self.find_inner::<true, false>(needle),
            (false, true) => self.find_inner::<false, true>(needle),
            (false, false) => self.find_inner::<false, false>(needle),
        }
    }
    #[inline]
    pub fn find_string(&self, needle: &crate::BitString) -> Option<usize> {
        let n = needle.as_bit_str();
        if self.start % WORD_BITS == 0 {
            self.find_inner::<true, true>(n)
        } else {
            self.find_inner::<false, true>(n)
        }
    }
    #[inline]
    pub fn rfind_str(&self, needle: BitStr<'_>) -> Option<usize> {
        let hs_a = self.start % WORD_BITS == 0;
        let nd_a = needle.start % WORD_BITS == 0;
        match (hs_a, nd_a) {
            (true, true) => self.rfind_inner::<true, true>(needle),
            (true, false) => self.rfind_inner::<true, false>(needle),
            (false, true) => self.rfind_inner::<false, true>(needle),
            (false, false) => self.rfind_inner::<false, false>(needle),
        }
    }
    #[inline]
    pub fn rfind_string(&self, needle: &crate::BitString) -> Option<usize> {
        let n = needle.as_bit_str();
        if self.start % WORD_BITS == 0 {
            self.rfind_inner::<true, true>(n)
        } else {
            self.rfind_inner::<false, true>(n)
        }
    }
}

#[cfg(test)]
mod tests_for_find;
