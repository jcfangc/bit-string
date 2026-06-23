use crate::traits::*;
use crate::{WORD_BITS, low_mask};

use crate::BitStr;

impl<'bs> BitStr<'bs> {
    /// Compare `needle` bits against `self` starting at `offset`.
    ///
    /// When `needle.start` is word-aligned, the fast path delegates to
    /// SIMD word-equality via [`BitsEq::eq_words`].  Otherwise falls
    /// back to a scalar word-at-a-time comparison.
    #[inline]
    pub(crate) fn bits_equal_at(&self, offset: usize, needle: &BitStr<'_>) -> bool {
        let n = needle.bit_len;
        if n == 0 {
            return true;
        }

        let hs_base = self.start + offset;
        let nd_base = needle.start;
        let hs_words = self.source.words();
        let nd_words = needle.source.words();

        // Sub-word fast path — a single 64-bit read on each side.
        if n <= WORD_BITS {
            let mask = low_mask(n);
            let h = hs_words.read_word_at(hs_base);
            let nd = nd_words.read_word_at(nd_base);
            return (h & mask) == (nd & mask);
        }

        // Multi-word, needle word-aligned — SIMD eq_words on haystack.
        if nd_base % WORD_BITS == 0 {
            let full_words = n / WORD_BITS;
            let nd_aligned = &nd_words[nd_base / WORD_BITS..];

            if !hs_words.eq_words(nd_aligned, full_words, hs_base) {
                return false;
            }

            let rem = n % WORD_BITS;
            if rem > 0 {
                let mask = low_mask(rem);
                let h = hs_words.read_word_at(hs_base + full_words * WORD_BITS);
                if (h & mask) != (nd_aligned[full_words] & mask) {
                    return false;
                }
            }

            return true;
        }

        // Both sides misaligned — scalar word-at-a-time (rare).
        let full_words = n / WORD_BITS;
        for i in 0..full_words {
            let pos = i * WORD_BITS;
            let h = hs_words.read_word_at(hs_base + pos);
            let nd = nd_words.read_word_at(nd_base + pos);
            if h != nd {
                return false;
            }
        }

        let rem = n % WORD_BITS;
        if rem > 0 {
            let mask = low_mask(rem);
            let pos = full_words * WORD_BITS;
            let h = hs_words.read_word_at(hs_base + pos);
            let nd = nd_words.read_word_at(nd_base + pos);
            if (h & mask) != (nd & mask) {
                return false;
            }
        }

        true
    }

    /// Returns `true` if `pattern` matches the bits starting at `index`.
    #[inline]
    pub fn matches_at(&self, index: usize, pattern: &BitStr<'_>) -> bool {
        if index > self.bit_len {
            return false;
        }
        if pattern.bit_len > self.bit_len - index {
            return false;
        }
        self.bits_equal_at(index, pattern)
    }

    /// Returns `true` if `prefix` is a prefix of `self`.
    #[inline]
    pub fn starts_with(&self, prefix: &BitStr<'_>) -> bool {
        self.matches_at(0, prefix)
    }

    /// Returns `true` if `suffix` is a suffix of `self`.
    #[inline]
    pub fn ends_with(&self, suffix: &BitStr<'_>) -> bool {
        if suffix.bit_len == 0 {
            return true;
        }
        if suffix.bit_len > self.bit_len {
            return false;
        }
        self.bits_equal_at(self.bit_len - suffix.bit_len, suffix)
    }
}

#[cfg(test)]
mod tests_for_matches_at;
