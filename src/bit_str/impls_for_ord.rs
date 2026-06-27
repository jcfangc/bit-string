use core::cmp::Ordering;

use crate::traits::*;
use crate::{WORD_BITS, low_mask};

use crate::BitStr;

impl<'bs> BitStr<'bs> {
    /// Lexicographic comparison of two bit strings.
    ///
    /// Compares bits from index 0 upward.  At the first differing bit, the
    /// side with a `1` is greater.  When all bits in the common prefix are
    /// equal, the longer bit string is greater.
    #[inline]
    pub fn cmp(&self, other: &BitStr<'bs>) -> Ordering {
        let common = self.bit_len.min(other.bit_len);
        if common == 0 {
            return self.bit_len.cmp(&other.bit_len);
        }

        let hs_words = self.source.words();
        let nd_words = other.source.words();
        let hs_base = self.start;
        let nd_base = other.start;
        let hs_aligned = hs_base % WORD_BITS == 0;
        let nd_aligned = nd_base % WORD_BITS == 0;

        let full = common / WORD_BITS;

        // Full-word comparison — follow the same dispatch pattern as
        // [`BitsEq::eq_words`]: SIMD when `other` is word-aligned.
        if nd_aligned {
            let nd_slice = &nd_words[nd_base / WORD_BITS..];
            if let Some(ord) = hs_words.cmp_words(nd_slice, full, hs_base) {
                return ord;
            }
        } else if hs_aligned {
            // Only `self` is aligned — swap so `other` becomes the
            // word-aligned reference, then reverse.
            let hs_slice = &hs_words[hs_base / WORD_BITS..];
            if let Some(ord) = nd_words.cmp_words(hs_slice, full, nd_base) {
                return ord.reverse();
            }
        } else {
            // Both sides misaligned — scalar word-at-a-time (rare).
            for i in 0..full {
                let pos = i * WORD_BITS;
                let a = hs_words.read_word_at(hs_base + pos);
                let b = nd_words.read_word_at(nd_base + pos);
                if a != b {
                    return a.bitwise_cmp(b);
                }
            }
        }

        // Partial tail word.
        let rem = common % WORD_BITS;
        if rem > 0 {
            let pos = full * WORD_BITS;
            let mask = low_mask(rem);
            let a = hs_words.read_word_at(hs_base + pos) & mask;
            let b = nd_words.read_word_at(nd_base + pos) & mask;
            if a != b {
                return a.bitwise_cmp(b);
            }
        }

        // Common prefix identical — longer wins.
        self.bit_len.cmp(&other.bit_len)
    }
}

impl PartialOrd for BitStr<'_> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BitStr<'_> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }
}

#[cfg(test)]
mod tests_for_ord;
