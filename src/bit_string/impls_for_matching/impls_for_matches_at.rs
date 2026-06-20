use crate::WORD_BITS;
use crate::funcs_for_bits::low_mask;

use super::*;

mod funcs_for_starts_with_core;

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

        // Word-aligned at position 0 — use SIMD word equality.
        if !funcs_for_starts_with_core::starts_with_words(sw, pw, full_words) {
            return false;
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
        if suffix.bit_len > self.bit_len {
            return false;
        }

        let start = self.bit_len - suffix.bit_len;
        let shift = start % WORD_BITS;
        let base_word = start / WORD_BITS;
        let sw: &[u64] = &self.words[base_word..];
        let pw = suffix.as_words();
        let full_words = suffix.bit_len / WORD_BITS;

        if !funcs_for_starts_with_core::ends_with_words(sw, pw, full_words, shift) {
            return false;
        }

        let rem = suffix.bit_len % WORD_BITS;
        if rem > 0 {
            let mask = low_mask(rem);
            let h = if shift == 0 {
                sw[full_words]
            } else {
                let w0 = sw[full_words];
                let w1 = sw.get(full_words + 1).copied().unwrap_or(0);
                (w0 >> shift) | (w1 << (WORD_BITS - shift))
            };
            if (h & mask) != (pw[full_words] & mask) {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests_for_matches_at;

#[cfg(test)]
mod tests_for_starts_with;

#[cfg(test)]
mod tests_for_ends_with;
