use crate::traits::*;
use crate::{WORD_BITS, low_mask};

use crate::BitStr;

impl<'bs> BitStr<'bs> {
    /// Compare `needle` bits against `self` starting at `offset`.
    ///
    /// When `HS_WORD_ALIGNED` is `true`, `self.start + offset` is
    /// guaranteed to be word-aligned.  When `ND_WORD_ALIGNED` is `true`,
    /// `needle.start` is guaranteed to be word-aligned.  Both compile-time
    /// signals eliminate runtime alignment checks.
    #[inline]
    pub(crate) fn bits_equal_at_inner<const HS_WORD_ALIGNED: bool, const ND_WORD_ALIGNED: bool>(
        &self,
        offset: usize,
        needle: BitStr<'_>,
    ) -> bool {
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
        // When ND_WORD_ALIGNED is true, the `nd_base % WORD_BITS == 0`
        // branch is unconditionally taken; LLVM eliminates the else.
        let nd_is_aligned = ND_WORD_ALIGNED || nd_base % WORD_BITS == 0;
        if nd_is_aligned {
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

    /// Public entry point — no compile-time alignment guarantees.
    #[inline]
    pub(crate) fn bits_equal_at(&self, offset: usize, needle: BitStr<'_>) -> bool {
        self.bits_equal_at_inner::<false, false>(offset, needle)
    }

    /// Returns `true` if `pattern` matches the bits starting at `index`.
    #[inline]
    pub fn matches_at(&self, index: usize, pattern: BitStr<'_>) -> bool {
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
    pub fn starts_with(&self, prefix: BitStr<'_>) -> bool {
        let hs_aligned = self.start % WORD_BITS == 0;
        let nd_aligned = prefix.start % WORD_BITS == 0;
        match (hs_aligned, nd_aligned) {
            (true, true) => self.matches_at_inner::<true, true>(0, prefix),
            (true, false) => self.matches_at_inner::<true, false>(0, prefix),
            (false, true) => self.matches_at_inner::<false, true>(0, prefix),
            (false, false) => self.matches_at_inner::<false, false>(0, prefix),
        }
    }

    /// Returns `true` if `suffix` is a suffix of `self`.
    #[inline]
    pub fn ends_with(&self, suffix: BitStr<'_>) -> bool {
        if suffix.bit_len == 0 {
            return true;
        }
        if suffix.bit_len > self.bit_len {
            return false;
        }
        self.bits_equal_at(self.bit_len - suffix.bit_len, suffix)
    }

    // -------------------------------------------------------------------
    // Inner helpers with alignment const-generics
    // -------------------------------------------------------------------

    /// Like [`matches_at`](Self::matches_at) but with compile-time
    /// alignment signals for both haystack and needle.
    #[inline]
    pub(crate) fn matches_at_inner<const HS_WORD_ALIGNED: bool, const ND_WORD_ALIGNED: bool>(
        &self,
        index: usize,
        pattern: BitStr<'_>,
    ) -> bool {
        if index > self.bit_len {
            return false;
        }
        if pattern.bit_len > self.bit_len - index {
            return false;
        }
        self.bits_equal_at_inner::<HS_WORD_ALIGNED, ND_WORD_ALIGNED>(index, pattern)
    }
}

#[cfg(test)]
mod tests_for_bits_equal_at;

#[cfg(test)]
mod tests_for_ends_with;

#[cfg(test)]
mod tests_for_matches_at;

#[cfg(test)]
mod tests_for_starts_with;
