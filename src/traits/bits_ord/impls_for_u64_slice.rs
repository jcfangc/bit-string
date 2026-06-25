use core::cmp::Ordering;

use crate::WORD_BITS;

use super::BitsOrd;
use super::funcs_for_cmp_aligned_core;
use super::funcs_for_cmp_unaligned_core;

impl BitsOrd for [u64] {
    #[inline]
    fn cmp_words(&self, other: &[u64], count: usize, offset: usize) -> Option<Ordering> {
        let shift = offset % WORD_BITS;
        let base = offset / WORD_BITS;
        let sw = &self[base..];

        if shift == 0 {
            funcs_for_cmp_aligned_core::cmp_aligned_words(sw, other, count)
        } else {
            funcs_for_cmp_unaligned_core::cmp_unaligned_words(sw, other, count, shift)
        }
    }
}
