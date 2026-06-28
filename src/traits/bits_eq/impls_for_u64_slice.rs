use crate::WORD_BITS;

use super::BitsEq;

impl BitsEq for [u64] {
    #[inline]
    fn eq_words<const HS_WORD_ALIGNED: bool>(
        &self,
        needle: &[u64],
        full_words: usize,
        haystack_shift: usize,
    ) -> bool {
        if HS_WORD_ALIGNED || haystack_shift == 0 {
            super::funcs_for_eq_words_aligned_core::eq_words_aligned(self, needle, full_words)
        } else {
            super::funcs_for_eq_words_unaligned_core::eq_words_unaligned(
                self,
                needle,
                full_words,
                haystack_shift,
            )
        }
    }
}
