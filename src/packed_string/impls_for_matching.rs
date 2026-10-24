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

    /// Returns whether `needle` occurs at a character boundary in this string.
    ///
    /// The supplied options are workload hints and do not change matching semantics.
    pub fn contains_with(&self, needle: &Self, _options: ContainsOptions) -> bool {
        (0..=self.char_len().saturating_sub(needle.char_len()))
            .any(|index| self.matches_at(index, needle))
    }

    /// Returns the character index, relative to this string, of the first occurrence of `needle`.
    ///
    /// The supplied options are workload hints and do not change matching semantics.
    pub fn find_with(&self, needle: &Self, _options: FindOptions) -> Option<usize> {
        (0..=self.char_len().saturating_sub(needle.char_len()))
            .find(|&index| self.matches_at(index, needle))
    }

    /// Returns the character index, relative to this string, of the last occurrence of `needle`.
    ///
    /// The supplied options are workload hints and do not change matching semantics.
    pub fn rfind_with(&self, needle: &Self, _options: FindOptions) -> Option<usize> {
        (0..=self.char_len().saturating_sub(needle.char_len()))
            .rev()
            .find(|&index| self.matches_at(index, needle))
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
