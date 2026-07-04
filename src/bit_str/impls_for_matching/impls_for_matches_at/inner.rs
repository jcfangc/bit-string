use crate::BitStr;
use crate::traits::*;
use crate::{WORD_BITS, low_mask};

impl<'bs> BitStr<'bs> {
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
        if n <= WORD_BITS {
            let mask = low_mask(n);
            let h = hs_words.read_word_at::<false>(hs_base);
            let nd = nd_words.read_word_at::<false>(nd_base);
            return (h & mask) == (nd & mask);
        }
        let nd_is_aligned = ND_WORD_ALIGNED || nd_base % WORD_BITS == 0;
        if nd_is_aligned {
            let full_words = n / WORD_BITS;
            let nd_aligned = &nd_words[nd_base / WORD_BITS..];
            let haystack = &hs_words[hs_base / WORD_BITS..];
            let ok = if HS_WORD_ALIGNED {
                haystack.eq_words::<true>(nd_aligned, full_words, 0)
            } else {
                haystack.eq_words::<false>(nd_aligned, full_words, hs_base % WORD_BITS)
            };
            if !ok {
                return false;
            }
            let rem = n % WORD_BITS;
            if rem > 0 {
                let mask = low_mask(rem);
                let h = hs_words.read_word_at::<false>(hs_base + full_words * WORD_BITS);
                if (h & mask) != (nd_aligned[full_words] & mask) {
                    return false;
                }
            }
            return true;
        }
        let full_words = n / WORD_BITS;
        for i in 0..full_words {
            let pos = i * WORD_BITS;
            let h = hs_words.read_word_at::<false>(hs_base + pos);
            let nd = nd_words.read_word_at::<false>(nd_base + pos);
            if h != nd {
                return false;
            }
        }
        let rem = n % WORD_BITS;
        if rem > 0 {
            let mask = low_mask(rem);
            let pos = full_words * WORD_BITS;
            let h = hs_words.read_word_at::<false>(hs_base + pos);
            let nd = nd_words.read_word_at::<false>(nd_base + pos);
            if (h & mask) != (nd & mask) {
                return false;
            }
        }
        true
    }

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

    #[inline]
    pub(crate) fn ends_with_inner<const HS_WORD_ALIGNED: bool, const ND_WORD_ALIGNED: bool>(
        &self,
        suffix: BitStr<'_>,
        offset: usize,
    ) -> bool {
        self.bits_equal_at_inner::<HS_WORD_ALIGNED, ND_WORD_ALIGNED>(offset, suffix)
    }

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
