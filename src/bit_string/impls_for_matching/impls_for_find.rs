use crate::WORD_BITS;
use crate::{ContainsOptions, FindOptions};

use super::*;

impl BitString {
    /// Returns whether `needle` occurs in this string using the supplied hints.
    #[inline]
    pub fn contains_str_with(&self, needle: crate::BitStr<'_>, _options: ContainsOptions) -> bool {
        let view = self.as_bit_str();
        if needle.start % WORD_BITS == 0 {
            view.contains_inner::<true, true>(needle)
        } else {
            view.contains_inner::<true, false>(needle)
        }
    }

    /// Returns whether the owned `needle` occurs in this string using the supplied hints.
    #[inline]
    pub fn contains_string_with(&self, needle: &BitString, _options: ContainsOptions) -> bool {
        self.as_bit_str()
            .contains_inner::<true, true>(needle.as_bit_str())
    }

    /// Returns the first bit index where `needle` occurs using the supplied hints.
    #[inline]
    pub fn find_str_with(&self, needle: crate::BitStr<'_>, _options: FindOptions) -> Option<usize> {
        let view = self.as_bit_str();
        if needle.start % WORD_BITS == 0 {
            view.find_inner::<true, true>(needle)
        } else {
            view.find_inner::<true, false>(needle)
        }
    }

    /// Returns the first bit index where the owned `needle` occurs using the supplied hints.
    #[inline]
    pub fn find_string_with(&self, needle: &BitString, _options: FindOptions) -> Option<usize> {
        self.as_bit_str()
            .find_inner::<true, true>(needle.as_bit_str())
    }

    /// Returns the last bit index where `needle` occurs using the supplied hints.
    #[inline]
    pub fn rfind_str_with(
        &self,
        needle: crate::BitStr<'_>,
        _options: FindOptions,
    ) -> Option<usize> {
        let view = self.as_bit_str();
        if needle.start % WORD_BITS == 0 {
            view.rfind_inner::<true, true>(needle)
        } else {
            view.rfind_inner::<true, false>(needle)
        }
    }

    /// Returns the last bit index where the owned `needle` occurs using the supplied hints.
    #[inline]
    pub fn rfind_string_with(&self, needle: &BitString, _options: FindOptions) -> Option<usize> {
        self.as_bit_str()
            .rfind_inner::<true, true>(needle.as_bit_str())
    }

    // -------------------------------------------------------------------
    // _str methods — needle is BitStr (hs is BitString-aligned)
    // -------------------------------------------------------------------

    /// Returns `true` if `needle` is contained within `self`.
    #[inline]
    pub fn contains_str(&self, needle: crate::BitStr<'_>) -> bool {
        self.contains_str_with(needle, ContainsOptions::default())
    }

    #[inline]
    pub fn find_str(&self, needle: crate::BitStr<'_>) -> Option<usize> {
        self.find_str_with(needle, FindOptions::default())
    }

    #[inline]
    pub fn rfind_str(&self, needle: crate::BitStr<'_>) -> Option<usize> {
        self.rfind_str_with(needle, FindOptions::default())
    }

    // -------------------------------------------------------------------
    // _string methods — both sides are BitString (double word-aligned)
    // -------------------------------------------------------------------

    /// Returns `true` if `needle` is contained within `self`.
    #[inline]
    pub fn contains_string(&self, needle: &BitString) -> bool {
        self.contains_string_with(needle, ContainsOptions::default())
    }

    #[inline]
    pub fn find_string(&self, needle: &BitString) -> Option<usize> {
        self.find_string_with(needle, FindOptions::default())
    }

    #[inline]
    pub fn rfind_string(&self, needle: &BitString) -> Option<usize> {
        self.rfind_string_with(needle, FindOptions::default())
    }
}
