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

    // -------------------------------------------------------------------
    // Public API
    // -------------------------------------------------------------------

    /// Returns `true` if `pattern` matches the bits starting at `index`.
    #[inline]
    pub fn matches_at_str(&self, index: usize, pattern: BitStr<'_>) -> bool {
        let hs_aligned = (self.start + index) % WORD_BITS == 0;
        let nd_aligned = pattern.start % WORD_BITS == 0;
        match (hs_aligned, nd_aligned) {
            (true, true) => self.matches_at_inner::<true, true>(index, pattern),
            (true, false) => self.matches_at_inner::<true, false>(index, pattern),
            (false, true) => self.matches_at_inner::<false, true>(index, pattern),
            (false, false) => self.matches_at_inner::<false, false>(index, pattern),
        }
    }

    /// `matches_at_str` when `pattern` is a [`BitString`](crate::BitString).
    #[inline]
    pub fn matches_at_string(&self, index: usize, pattern: &crate::BitString) -> bool {
        let p = pattern.as_bit_str();
        if (self.start + index) % WORD_BITS == 0 {
            self.matches_at_inner::<true, true>(index, p)
        } else {
            self.matches_at_inner::<false, true>(index, p)
        }
    }

    /// Returns `true` if `prefix` is a prefix of `self`.
    #[inline]
    pub fn starts_with_str(&self, prefix: BitStr<'_>) -> bool {
        let hs_aligned = self.start % WORD_BITS == 0;
        let nd_aligned = prefix.start % WORD_BITS == 0;
        match (hs_aligned, nd_aligned) {
            (true, true) => self.starts_with_inner::<true, true>(prefix),
            (true, false) => self.starts_with_inner::<true, false>(prefix),
            (false, true) => self.starts_with_inner::<false, true>(prefix),
            (false, false) => self.starts_with_inner::<false, false>(prefix),
        }
    }

    /// Returns `true` if `suffix` is a suffix of `self`.
    #[inline]
    pub fn ends_with_str(&self, suffix: BitStr<'_>) -> bool {
        if suffix.bit_len == 0 {
            return true;
        }
        if suffix.bit_len > self.bit_len {
            return false;
        }
        let hs_aligned = self.start % WORD_BITS == 0;
        let nd_aligned = suffix.start % WORD_BITS == 0;
        let offset = self.bit_len - suffix.bit_len;
        match (hs_aligned, nd_aligned) {
            (true, true) => self.ends_with_inner::<true, true>(suffix, offset),
            (true, false) => self.ends_with_inner::<true, false>(suffix, offset),
            (false, true) => self.ends_with_inner::<false, true>(suffix, offset),
            (false, false) => self.ends_with_inner::<false, false>(suffix, offset),
        }
    }

    // -------------------------------------------------------------------
    // _string methods — argument is &BitString (word-aligned)
    // -------------------------------------------------------------------

    /// `starts_with` when `prefix` is a [`BitString`](crate::BitString)
    /// (always word-aligned).  Only the haystack alignment is checked.
    #[inline]
    pub fn starts_with_string(&self, prefix: &crate::BitString) -> bool {
        let p = prefix.as_bit_str();
        if self.start % WORD_BITS == 0 {
            self.starts_with_inner::<true, true>(p)
        } else {
            self.starts_with_inner::<false, true>(p)
        }
    }

    /// `ends_with` when `suffix` is a [`BitString`](crate::BitString).
    #[inline]
    pub fn ends_with_string(&self, suffix: &crate::BitString) -> bool {
        let s = suffix.as_bit_str();
        if s.bit_len == 0 {
            return true;
        }
        if s.bit_len > self.bit_len {
            return false;
        }
        let offset = self.bit_len - s.bit_len;
        if self.start % WORD_BITS == 0 {
            self.ends_with_inner::<true, true>(s, offset)
        } else {
            self.ends_with_inner::<false, true>(s, offset)
        }
    }

    // -------------------------------------------------------------------
    // Inner helpers with alignment const-generics
    // -------------------------------------------------------------------

    /// `starts_with` with compile-time alignment signals.
    #[inline]
    pub(crate) fn starts_with_inner<const HS_WORD_ALIGNED: bool, const ND_WORD_ALIGNED: bool>(
        &self,
        prefix: BitStr<'_>,
    ) -> bool {
        if prefix.bit_len > self.bit_len {
            return false;
        }
        self.bits_equal_at_inner::<HS_WORD_ALIGNED, ND_WORD_ALIGNED>(0, prefix)
    }

    /// `ends_with` with compile-time alignment signals.
    #[inline]
    pub(crate) fn ends_with_inner<const HS_WORD_ALIGNED: bool, const ND_WORD_ALIGNED: bool>(
        &self,
        suffix: BitStr<'_>,
        offset: usize,
    ) -> bool {
        self.bits_equal_at_inner::<HS_WORD_ALIGNED, ND_WORD_ALIGNED>(offset, suffix)
    }

    /// Like `matches_at_str` but with compile-time alignment signals.
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
