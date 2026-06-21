use super::BitsEq;

impl BitsEq for [u64] {
    #[inline]
    fn eq_words(&self, other: &[u64], count: usize) -> bool {
        super::funcs_for_eq_words_core::eq_words(self, other, count)
    }

    #[inline]
    fn eq_words_shifted(&self, other: &[u64], count: usize, shift: usize) -> bool {
        super::funcs_for_eq_words_shifted_core::eq_words_shifted(self, other, count, shift)
    }
}
