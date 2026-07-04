use crate::BitStr;
use crate::traits::*;
use crate::{WORD_BITS, low_mask};
use core::cmp::Ordering;

impl<'bs> BitStr<'bs> {
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
            let hs_slice = &hs_words[hs_base / WORD_BITS..];
            let nd_slice = &nd_words[nd_base / WORD_BITS..];
            if let Some(ord) = nd_slice.cmp_words::<false>(hs_slice, full, nd_base % WORD_BITS) {
                return ord.reverse();
            }
        } else {
            for i in 0..full {
                let pos = i * WORD_BITS;
                let a = hs_words.read_word_at::<false>(hs_base + pos);
                let b = nd_words.read_word_at::<false>(nd_base + pos);
                if a != b {
                    return a.bitwise_cmp(b);
                }
            }
        }
        let rem = common % WORD_BITS;
        if rem > 0 {
            let pos = full * WORD_BITS;
            let mask = low_mask(rem);
            let a = hs_words.read_word_at::<false>(hs_base + pos) & mask;
            let b = nd_words.read_word_at::<false>(nd_base + pos) & mask;
            if a != b {
                return a.bitwise_cmp(b);
            }
        }
        self.bit_len.cmp(&other.bit_len)
    }
}
