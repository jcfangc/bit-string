use crate::WORD_BITS;
use crate::funcs_for_bits::low_mask;

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
        if prefix.bit_len > self.bit_len {
            return false;
        }

        let pw = prefix.as_words();
        let sw: &[u64] = &self.words;
        let full_words = prefix.bit_len / WORD_BITS;

        // Prefix starts at position 0 — both haystack and needle are
        // word-aligned, so direct u64 comparison skips read_word_at overhead.
        for i in 0..full_words {
            if sw[i] != pw[i] {
                return false;
            }
        }

        let rem = prefix.bit_len % WORD_BITS;
        if rem > 0 {
            let mask = low_mask(rem);
            if (sw[full_words] & mask) != (pw[full_words] & mask) {
                return false;
            }
        }

        true
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
