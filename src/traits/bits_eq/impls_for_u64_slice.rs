use crate::WORD_BITS;

use super::BitsEq;

impl BitsEq for [u64] {
    #[inline]
    fn eq_words(&self, other: &[u64], count: usize, offset: usize) -> bool {
        let shift = offset % WORD_BITS;
        let base = offset / WORD_BITS;
        let sw = &self[base..];

        if shift == 0 {
            super::funcs_for_eq_words_aligned_core::eq_words_aligned(sw, other, count)
        } else {
            super::funcs_for_eq_words_unaligned_core::eq_words_unaligned(sw, other, count, shift)
        }
    }
}
