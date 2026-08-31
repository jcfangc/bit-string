use core::cmp::Ordering;

use crate::WORD_BITS;
use crate::traits::WordOrd;

#[inline]
pub(super) fn cmp_unaligned(
    src: &[u64],
    other: &[u64],
    count: usize,
    shift: usize,
) -> Option<Ordering> {
    for i in 0..count {
        let w0 = src[i];
        let w1 = src[i + 1];
        let window = (w0 >> shift) | (w1 << (WORD_BITS - shift));
        if window != other[i] {
            return Some(WordOrd::bitwise_cmp(window, other[i]));
        }
    }
    None
}
