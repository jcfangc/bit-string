use crate::bit_string::traits::*;

use super::*;

impl BitString {
    pub fn matches_at(&self, index: usize, pattern: &Self) -> bool {
        if index > self.bit_len {
            return false;
        }

        if pattern.bit_len > self.bit_len - index {
            return false;
        }

        bits_equal_at(self, index, pattern)
    }

    #[inline]
    pub fn starts_with(&self, prefix: &Self) -> bool {
        self.matches_at(0, prefix)
    }

    #[inline]
    pub fn ends_with(&self, suffix: &Self) -> bool {
        suffix.bit_len <= self.bit_len && self.matches_at(self.bit_len - suffix.bit_len, suffix)
    }
}

#[cfg(test)]
mod tests_for_matches_at;

#[cfg(test)]
mod tests_for_starts_with;

#[cfg(test)]
mod tests_for_ends_with;
