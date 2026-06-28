use crate::traits::*;
use crate::{SMALL_WORDS, WORD_BITS};

use crate::BitStr;

impl<'bs> BitStr<'bs> {
    /// Returns `true` if `needle` is contained within `self`.
    #[inline]
    pub fn contains_str(&self, needle: BitStr<'_>) -> bool {
        let hs_aligned = self.start % WORD_BITS == 0;
        let nd_aligned = needle.start % WORD_BITS == 0;
        match (hs_aligned, nd_aligned) {
            (true, true) => self.contains_inner::<true, true>(needle),
            (true, false) => self.contains_inner::<true, false>(needle),
            (false, true) => self.contains_inner::<false, true>(needle),
            (false, false) => self.contains_inner::<false, false>(needle),
        }
    }

    /// Returns the index of the first occurrence of `needle`, or `None`.
    #[inline]
    pub fn find_str(&self, needle: BitStr<'_>) -> Option<usize> {
        let hs_a = self.start % WORD_BITS == 0;
        let nd_a = needle.start % WORD_BITS == 0;
        match (hs_a, nd_a) {
            (true, true) => self.find_inner::<true, true>(needle),
            (true, false) => self.find_inner::<true, false>(needle),
            (false, true) => self.find_inner::<false, true>(needle),
            (false, false) => self.find_inner::<false, false>(needle),
        }
    }

    /// `find_str` when `needle` is a [`BitString`](crate::BitString).
    #[inline]
    pub fn find_string(&self, needle: &crate::BitString) -> Option<usize> {
        let n = needle.as_bit_str();
        if self.start % WORD_BITS == 0 {
            self.find_inner::<true, true>(n)
        } else {
            self.find_inner::<false, true>(n)
        }
    }

    /// `find` with compile-time alignment signals.
    #[inline]
    pub(crate) fn find_inner<const WORD_ALIGNED: bool, const ND_WORD_ALIGNED: bool>(
        &self,
        needle: BitStr<'_>,
    ) -> Option<usize> {
        if needle.bit_len == 0 {
            return Some(0);
        }
        if needle.bit_len > self.bit_len {
            return None;
        }

        let words = self.source.words();
        let sw = self.start / WORD_BITS;
        let so = self.start % WORD_BITS;
        let needle_words = needle.source.words();
        let needle_len = needle.bit_len;

        // Word-aligned fast path.
        if WORD_ALIGNED || so == 0 {
            return words[sw..].find_first_word(
                self.bit_len,
                needle_words,
                needle_len,
                &mut |pos| self.bits_equal_at_inner::<false, ND_WORD_ALIGNED>(pos, needle),
            );
        }

        // Unaligned: scan the first partial word, then SIMD for the rest.
        let first_bits = (WORD_BITS - so).min(self.bit_len);
        let max = first_bits.min(self.bit_len.saturating_sub(needle_len));
        for p in 0..=max {
            if self.bits_equal_at_inner::<false, ND_WORD_ALIGNED>(p, needle) {
                return Some(p);
            }
        }

        let remaining = self.bit_len - first_bits;
        if remaining == 0 {
            return None;
        }

        let aligned = &words[sw + 1..];

        // Quick rejection before the more expensive word-outer scan.
        if aligned.len() >= SMALL_WORDS
            && !aligned
                .find_any_candidate(remaining, needle_words, needle_len, &mut |pos| {
                    self.bits_equal_at_inner::<false, ND_WORD_ALIGNED>(pos + first_bits, needle)
                })
                .is_some()
        {
            return None;
        }

        aligned
            .find_first_word(remaining, needle_words, needle_len, &mut |pos| {
                self.bits_equal_at_inner::<false, ND_WORD_ALIGNED>(pos + first_bits, needle)
            })
            .map(|pos| pos + first_bits)
    }

    /// Returns the index of the last occurrence of `needle`, or `None`.
    #[inline]
    pub fn rfind_str(&self, needle: BitStr<'_>) -> Option<usize> {
        let hs_a = self.start % WORD_BITS == 0;
        let nd_a = needle.start % WORD_BITS == 0;
        match (hs_a, nd_a) {
            (true, true) => self.rfind_inner::<true, true>(needle),
            (true, false) => self.rfind_inner::<true, false>(needle),
            (false, true) => self.rfind_inner::<false, true>(needle),
            (false, false) => self.rfind_inner::<false, false>(needle),
        }
    }

    /// `rfind_str` when `needle` is a [`BitString`](crate::BitString).
    #[inline]
    pub fn rfind_string(&self, needle: &crate::BitString) -> Option<usize> {
        let n = needle.as_bit_str();
        if self.start % WORD_BITS == 0 {
            self.rfind_inner::<true, true>(n)
        } else {
            self.rfind_inner::<false, true>(n)
        }
    }

    /// `rfind` with compile-time alignment signals.
    #[inline]
    pub(crate) fn rfind_inner<const WORD_ALIGNED: bool, const ND_WORD_ALIGNED: bool>(
        &self,
        needle: BitStr<'_>,
    ) -> Option<usize> {
        if needle.bit_len == 0 {
            return Some(self.bit_len);
        }
        if needle.bit_len > self.bit_len {
            return None;
        }

        let words = self.source.words();
        let sw = self.start / WORD_BITS;
        let so = self.start % WORD_BITS;
        let needle_words = needle.source.words();
        let needle_len = needle.bit_len;

        // Word-aligned fast path.
        if WORD_ALIGNED || so == 0 {
            return words[sw..].find_last_word(
                self.bit_len,
                needle_words,
                needle_len,
                &mut |pos| self.bits_equal_at_inner::<false, ND_WORD_ALIGNED>(pos, needle),
            );
        }

        // Unaligned: SIMD on the aligned remainder first (reverse).
        let first_bits = (WORD_BITS - so).min(self.bit_len);
        let remaining = self.bit_len - first_bits;

        if remaining > 0 {
            let aligned = &words[sw + 1..];

            let maybe_candidate = aligned.len() < SMALL_WORDS
                || aligned
                    .find_any_candidate(remaining, needle_words, needle_len, &mut |pos| {
                        self.bits_equal_at_inner::<false, ND_WORD_ALIGNED>(pos + first_bits, needle)
                    })
                    .is_some();

            if maybe_candidate {
                if let Some(pos) =
                    aligned.find_last_word(remaining, needle_words, needle_len, &mut |pos| {
                        self.bits_equal_at_inner::<false, ND_WORD_ALIGNED>(pos + first_bits, needle)
                    })
                {
                    return Some(pos + first_bits);
                }
            }
        }

        // Check the first partial word.
        let max = first_bits.min(self.bit_len.saturating_sub(needle_len));
        for p in (0..=max).rev() {
            if self.bits_equal_at_inner::<false, ND_WORD_ALIGNED>(p, needle) {
                return Some(p);
            }
        }

        None
    }

    // -------------------------------------------------------------------
    // _string methods — argument is &BitString (word-aligned)
    // -------------------------------------------------------------------

    /// `contains` when `needle` is a [`BitString`](crate::BitString).
    #[inline]
    pub fn contains_string(&self, needle: &crate::BitString) -> bool {
        let n = needle.as_bit_str();
        if self.start % WORD_BITS == 0 {
            self.contains_inner::<true, true>(n)
        } else {
            self.contains_inner::<false, true>(n)
        }
    }

    // -------------------------------------------------------------------
    // Inner helpers with alignment const-generics
    // -------------------------------------------------------------------

    /// `contains` with compile-time alignment signals.
    ///
    /// When `HS_WORD_ALIGNED` is `true`, the `start % WORD_BITS == 0`
    /// branch is eliminated and we go straight to the aligned SIMD path.
    #[inline]
    pub(crate) fn contains_inner<const HS_WORD_ALIGNED: bool, const ND_WORD_ALIGNED: bool>(
        &self,
        needle: BitStr<'_>,
    ) -> bool {
        if needle.bit_len == 0 {
            return true;
        }
        if needle.bit_len > self.bit_len {
            return false;
        }

        let words = self.source.words();
        let sw = self.start / WORD_BITS;
        let so = self.start % WORD_BITS;
        let needle_words = needle.source.words();
        let needle_len = needle.bit_len;

        // When HS_WORD_ALIGNED, so == 0 — LLVM eliminates the unaligned path.
        if !HS_WORD_ALIGNED && so != 0 {
            let first_bits = (WORD_BITS - so).min(self.bit_len);
            let max = first_bits.min(self.bit_len.saturating_sub(needle_len));
            for p in 0..=max {
                if
                // HS_WORD_ALIGNED is always false here — the candidate
                // position `p` varies, so hs_base % WORD_BITS is not
                // known at compile time.  Only needle alignment is fixed.
                self.bits_equal_at_inner::<false, ND_WORD_ALIGNED>(p, needle) {
                    return true;
                }
            }
            let remaining = self.bit_len - first_bits;
            if remaining == 0 {
                return false;
            }
            let aligned = &words[sw + 1..];
            return aligned
                .find_any_candidate(remaining, needle_words, needle_len, &mut |pos| {
                    // HS_WORD_ALIGNED is always false here — the candidate
                    // position `p` varies, so hs_base % WORD_BITS is not
                    // known at compile time.  Only needle alignment is fixed.
                    self.bits_equal_at_inner::<false, ND_WORD_ALIGNED>(pos + first_bits, needle)
                })
                .is_some();
        }

        // Word-aligned: full SIMD on the relevant suffix.
        words[sw..]
            .find_any_candidate(self.bit_len, needle_words, needle_len, &mut |pos| {
                // HS_WORD_ALIGNED is always false here — the candidate
                // position `p` varies, so hs_base % WORD_BITS is not
                // known at compile time.  Only needle alignment is fixed.
                self.bits_equal_at_inner::<false, ND_WORD_ALIGNED>(pos, needle)
            })
            .is_some()
    }
}

#[cfg(test)]
mod tests_for_find;
