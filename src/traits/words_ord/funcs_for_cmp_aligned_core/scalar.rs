use core::cmp::Ordering;

use crate::traits::WordOrd;

#[inline]
pub(super) fn cmp_aligned(src: &[u64], other: &[u64], count: usize) -> Option<Ordering> {
    for i in 0..count {
        if src[i] != other[i] {
            return Some(WordOrd::bitwise_cmp(src[i], other[i]));
        }
    }
    None
}
