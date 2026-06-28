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
    pub fn cmp_str(&self, other: &BitStr<'bs>) -> Ordering {
        let hs_aligned = self.start % WORD_BITS == 0;
        let nd_aligned = other.start % WORD_BITS == 0;
        match (hs_aligned, nd_aligned) {
            (true, true) => self.cmp_inner::<true, true>(other),
            (true, false) => self.cmp_inner::<true, false>(other),
            (false, true) => self.cmp_inner::<false, true>(other),
            (false, false) => self.cmp_inner::<false, false>(other),
        }
    }

    /// `cmp_str` when `other` is a [`BitString`](crate::BitString)
    /// (always word-aligned).
    #[inline]
    pub fn cmp_string(&self, other: &crate::BitString) -> Ordering {
        let o = other.as_bit_str();
        if self.start % WORD_BITS == 0 {
            self.cmp_inner::<true, true>(&o)
        } else {
            self.cmp_inner::<false, true>(&o)
        }
    }

    /// `cmp` with compile-time alignment signals.
    #[inline]
    pub(crate) fn cmp_inner<const HS_WORD_ALIGNED: bool, const ND_WORD_ALIGNED: bool>(
        &self,
        other: &BitStr<'bs>,
    ) -> Ordering {
        let common = self.bit_len.min(other.bit_len);
        if common == 0 {
            return self.bit_len.cmp(&other.bit_len);
        }

        let hs_words = self.source.words();
        let nd_words = other.source.words();
        let hs_base = self.start;
        let nd_base = other.start;

        let full = common / WORD_BITS;

        // Full-word comparison — follow the same dispatch pattern as
        // [`BitsEq::eq_words`]: SIMD when `other` is word-aligned.
        let nd_is_aligned = ND_WORD_ALIGNED || nd_base % WORD_BITS == 0;
        if nd_is_aligned {
            let nd_slice = &nd_words[nd_base / WORD_BITS..];
            let hs_slice = &hs_words[hs_base / WORD_BITS..];
            let ok = if HS_WORD_ALIGNED {
                hs_slice.cmp_words::<true>(nd_slice, full, 0)
            } else {
                hs_slice.cmp_words::<false>(nd_slice, full, hs_base % WORD_BITS)
            };
            if let Some(ord) = ok {
                return ord;
            }
        } else if HS_WORD_ALIGNED || hs_base % WORD_BITS == 0 {
            // Only `self` is aligned — swap so `other` becomes the
            // word-aligned reference, then reverse.
            let hs_slice = &hs_words[hs_base / WORD_BITS..];
            let nd_slice = &nd_words[nd_base / WORD_BITS..];
            if let Some(ord) = nd_slice.cmp_words::<false>(hs_slice, full, nd_base % WORD_BITS) {
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
        Some(self.cmp_str(other))
    }
}

impl Ord for BitStr<'_> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp_str(other)
    }
}

#[cfg(test)]
mod tests_for_ord;
