use crate::BitStr;
use crate::WORD_BITS;
use crate::{ContainsOptions, FindOptions};

mod inner;

impl<'bs> BitStr<'bs> {
    /// Returns whether `needle` occurs in this view using the supplied hints.
    #[inline]
    pub fn contains_str_with(&self, needle: BitStr<'_>, _options: ContainsOptions) -> bool {
        let hs_aligned = self.start % WORD_BITS == 0;
        let nd_aligned = needle.start % WORD_BITS == 0;
        match (hs_aligned, nd_aligned) {
            (true, true) => self.contains_inner::<true, true>(needle),
            (true, false) => self.contains_inner::<true, false>(needle),
            (false, true) => self.contains_inner::<false, true>(needle),
            (false, false) => self.contains_inner::<false, false>(needle),
        }
    }
    /// Returns whether `needle` occurs in this view using the supplied hints.
    #[inline]
    pub fn contains_string_with(
        &self,
        needle: &crate::BitString,
        _options: ContainsOptions,
    ) -> bool {
        let n = needle.as_bit_str();
        if self.start % WORD_BITS == 0 {
            self.contains_inner::<true, true>(n)
        } else {
            self.contains_inner::<false, true>(n)
        }
    }
    /// Returns the first matching bit index using the supplied hints.
    #[inline]
    pub fn find_str_with(&self, needle: BitStr<'_>, _options: FindOptions) -> Option<usize> {
        let hs_a = self.start % WORD_BITS == 0;
        let nd_a = needle.start % WORD_BITS == 0;
        match (hs_a, nd_a) {
            (true, true) => self.find_inner::<true, true>(needle),
            (true, false) => self.find_inner::<true, false>(needle),
            (false, true) => self.find_inner::<false, true>(needle),
            (false, false) => self.find_inner::<false, false>(needle),
        }
    }
    /// Returns the first matching bit index using the supplied hints.
    #[inline]
    pub fn find_string_with(
        &self,
        needle: &crate::BitString,
        _options: FindOptions,
    ) -> Option<usize> {
        let n = needle.as_bit_str();
        if self.start % WORD_BITS == 0 {
            self.find_inner::<true, true>(n)
        } else {
            self.find_inner::<false, true>(n)
        }
    }
    /// Returns the last matching bit index using the supplied hints.
    #[inline]
    pub fn rfind_str_with(&self, needle: BitStr<'_>, _options: FindOptions) -> Option<usize> {
        let hs_a = self.start % WORD_BITS == 0;
        let nd_a = needle.start % WORD_BITS == 0;
        match (hs_a, nd_a) {
            (true, true) => self.rfind_inner::<true, true>(needle),
            (true, false) => self.rfind_inner::<true, false>(needle),
            (false, true) => self.rfind_inner::<false, true>(needle),
            (false, false) => self.rfind_inner::<false, false>(needle),
        }
    }
    /// Returns the last matching bit index using the supplied hints.
    #[inline]
    pub fn rfind_string_with(
        &self,
        needle: &crate::BitString,
        _options: FindOptions,
    ) -> Option<usize> {
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
