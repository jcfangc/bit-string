use super::*;

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

    pub fn contains(&self, needle: &Self) -> bool {
        (0..=self.char_len().saturating_sub(needle.char_len()))
            .any(|index| self.matches_at(index, needle))
    }

    pub fn find(&self, needle: &Self) -> Option<usize> {
        (0..=self.char_len().saturating_sub(needle.char_len()))
            .find(|&index| self.matches_at(index, needle))
    }

    pub fn rfind(&self, needle: &Self) -> Option<usize> {
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
