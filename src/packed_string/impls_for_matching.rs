use super::*;
use crate::{ContainsOptions, FindOptions};

impl<C, const BITS: u8> PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    pub fn matches_at(&self, index: usize, pattern: &Self) -> bool {
        let Some(bit_index) = index.checked_mul(usize::from(BITS)) else {
            return false;
        };
        if index > self.char_len() {
            return false;
        }
        self.bits.matches_at_string(bit_index, &pattern.bits)
    }

    pub fn starts_with(&self, prefix: &Self) -> bool {
        self.bits.starts_with_string(&prefix.bits)
    }

    pub fn ends_with(&self, suffix: &Self) -> bool {
        self.bits.ends_with_string(&suffix.bits)
    }

    /// Returns whether `needle` occurs in this string using the supplied hints.
    pub fn contains_with(&self, needle: &Self, options: ContainsOptions) -> bool {
        if options.anchor() != crate::AnchorHint::Suffix
            && options.location() != crate::LocationHint::NearEnd
        {
            let needle_len = needle.char_len();
            let haystack_len = self.char_len();
            return needle_len <= haystack_len
                && (0..=haystack_len - needle_len).any(|index| self.matches_at(index, needle));
        }

        self.as_packed_str()
            .contains_with(needle.as_packed_str(), options)
    }

    /// Returns the first matching character index using the supplied hints.
    pub fn find_with(&self, needle: &Self, options: FindOptions) -> Option<usize> {
        self.as_packed_str()
            .find_with(needle.as_packed_str(), options)
    }

    /// Returns the last matching character index using the supplied hints.
    pub fn rfind_with(&self, needle: &Self, options: FindOptions) -> Option<usize> {
        self.as_packed_str()
            .rfind_with(needle.as_packed_str(), options)
    }

    pub fn strip_prefix(&self, prefix: &Self) -> Option<Self> {
        self.bits
            .strip_prefix_string(&prefix.bits)
            .map(Self::from_valid_bits)
    }

    pub fn strip_suffix(&self, suffix: &Self) -> Option<Self> {
        self.bits
            .strip_suffix_string(&suffix.bits)
            .map(Self::from_valid_bits)
    }
}
