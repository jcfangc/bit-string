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
        super::funcs_for_eq_words_core::eq_words::<HS_WORD_ALIGNED>(
            self,
            needle,
            full_words,
            haystack_shift,
        )
    }
}
