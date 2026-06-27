use core::cmp::Ordering;

/// Lexicographic comparison of two `u64` values (LSB-first bit order).
///
/// This is the word-level primitive used by [`BitsOrd`](super::BitsOrd)
/// to resolve ordering once the first differing word has been found.
pub(crate) trait BitOrd {
    fn bitwise_cmp(self, other: Self) -> Ordering;
}

impl BitOrd for u64 {
    #[inline]
    fn bitwise_cmp(self, other: u64) -> Ordering {
        debug_assert!(self != other);
        let diff = self ^ other;
        let first = diff.trailing_zeros();
        let a_bit = (self >> first) & 1;
        let b_bit = (other >> first) & 1;
        a_bit.cmp(&b_bit)
    }
}
