use super::BitsEq;

impl BitsEq for [u64] {
    #[inline]
    fn eq_words(&self, other: &[u64], count: usize, shift: usize) -> bool {
        if shift == 0 {
            super::funcs_for_eq_words_aligned_core::eq_words_aligned(self, other, count)
        } else {
            super::funcs_for_eq_words_unaligned_core::eq_words_unaligned(self, other, count, shift)
        }
    }
}
