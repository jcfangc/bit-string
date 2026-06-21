use crate::SMALL_WORDS;
use crate::WORD_BITS;
use crate::funcs_for_bits::low_mask;

use super::*;

impl BitString {
    /// Compare `needle` bits against `self` starting at `offset`.
    ///
    /// For word-aligned offsets, the full words are compared via
    /// [`BitsEq::eq_words`].  For unaligned offsets, shifted 64-bit
    /// windows are computed via [`BitsEq::eq_words_shifted`].
    /// Short patterns fall back to scalar.
    #[inline]
    pub(crate) fn bits_equal_at(&self, offset: usize, needle: &Self) -> bool {
        let needle_bits = needle.bit_len;
        if needle_bits == 0 {
            return true;
        }
        let needle_words = needle.as_words();

        // Sub-word fast path: the entire pattern fits in one u64 —
        // single read + mask avoids all branching and loop overhead.
        if needle_bits <= WORD_BITS {
            let h = self.words.read_word_at(offset);
            let mask = low_mask(needle_bits);
            return (h & mask) == (needle_words[0] & mask);
        }

        let full_words = needle_bits / WORD_BITS;

        if full_words >= SMALL_WORDS {
            let shift = offset % WORD_BITS;
            let base_word = offset / WORD_BITS;
            let sw: &[u64] = &self.words[base_word..];

            if shift == 0 {
                if !sw.eq_words(needle_words, full_words) {
                    return false;
                }
            } else {
                if !sw.eq_words_shifted(needle_words, full_words, shift) {
                    return false;
                }
            }
        } else {
            for i in 0..full_words {
                let h = self.words.read_word_at(offset + i * WORD_BITS);
                if h != needle_words[i] {
                    return false;
                }
            }
        }

        let rem_bits = needle_bits % WORD_BITS;
        if rem_bits > 0 {
            let mask = low_mask(rem_bits);
            let h = self.words.read_word_at(offset + full_words * WORD_BITS);
            if (h & mask) != (needle_words[full_words] & mask) {
                return false;
            }
        }

        true
    }

    /// Returns `true` if `pattern` matches the bits starting at `index`.
    ///
    /// Delegates to [`bits_equal_at`] which uses SIMD word-equality for
    /// long patterns and scalar comparison for short ones.
    #[inline]
    pub fn matches_at(&self, index: usize, pattern: &Self) -> bool {
        if index > self.bit_len {
            return false;
        }

        if pattern.bit_len > self.bit_len - index {
            return false;
        }

        self.bits_equal_at(index, pattern)
    }

    /// Returns `true` if `prefix` is a prefix of `self`.
    ///
    /// This is equivalent to [`matches_at`]`(0, prefix)` but optimized for
    /// the word-aligned position-0 case.
    #[inline]
    pub fn starts_with(&self, prefix: &Self) -> bool {
        if prefix.bit_len == 0 {
            return true;
        }
        if prefix.bit_len > self.bit_len {
            return false;
        }

        let pw = prefix.as_words();
        let sw: &[u64] = &self.words;

        // Sub-word fast path: one u64 read + mask.
        if prefix.bit_len <= WORD_BITS {
            let mask = low_mask(prefix.bit_len);
            return (sw[0] & mask) == (pw[0] & mask);
        }

        let full_words = prefix.bit_len / WORD_BITS;

        // Word-aligned at position 0 — use SIMD word equality.
        if !sw.eq_words(pw, full_words) {
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

    /// Returns `true` if `suffix` is a suffix of `self`.
    #[inline]
    pub fn ends_with(&self, suffix: &Self) -> bool {
        if suffix.bit_len == 0 {
            return true;
        }
        if suffix.bit_len > self.bit_len {
            return false;
        }

        let start = self.bit_len - suffix.bit_len;
        let shift = start % WORD_BITS;
        let base_word = start / WORD_BITS;
        let sw: &[u64] = &self.words[base_word..];
        let pw = suffix.as_words();

        // Sub-word fast path: one 64-bit window + mask.
        if suffix.bit_len <= WORD_BITS {
            let h = if shift == 0 {
                sw[0]
            } else {
                let w0 = sw[0];
                let w1 = sw.get(1).copied().unwrap_or(0);
                (w0 >> shift) | (w1 << (WORD_BITS - shift))
            };
            let mask = low_mask(suffix.bit_len);
            return (h & mask) == (pw[0] & mask);
        }

        let full_words = suffix.bit_len / WORD_BITS;

        if !sw.eq_words_shifted(pw, full_words, shift) {
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
