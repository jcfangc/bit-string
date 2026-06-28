use core::cmp::Ordering;

use super::WordsOrd;
use super::funcs_for_cmp_aligned_core;
use super::funcs_for_cmp_unaligned_core;

impl WordsOrd for [u64] {
    #[inline]
    fn cmp_words<const HS_WORD_ALIGNED: bool>(
        &self,
        needle: &[u64],
        full_words: usize,
        haystack_shift: usize,
    ) -> Option<Ordering> {
        if HS_WORD_ALIGNED || haystack_shift == 0 {
            funcs_for_cmp_aligned_core::cmp_aligned_words(self, needle, full_words)
        } else {
            funcs_for_cmp_unaligned_core::cmp_unaligned_words(
                self,
                needle,
                full_words,
                haystack_shift,
            )
        }
    }
}
