use core::cmp::Ordering;

use crate::BitString;
use crate::WORD_BITS;

impl BitString {
    /// Lexicographic comparison against a [`BitStr`](crate::BitStr).
    #[inline]
    pub fn cmp_str(&self, other: &crate::BitStr<'_>) -> Ordering {
        let view = self.as_bit_str();
        if other.start % WORD_BITS == 0 {
            view.cmp_inner::<true, true>(other)
        } else {
            view.cmp_inner::<true, false>(other)
        }
    }

    /// Lexicographic comparison — both sides are `BitString`
    /// (double word-aligned, zero runtime alignment checks).
    #[inline]
    pub fn cmp_string(&self, other: &BitString) -> Ordering {
        self.as_bit_str()
            .cmp_inner::<true, true>(&other.as_bit_str())
    }
}

impl PartialOrd for BitString {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp_string(other))
    }
}

impl Ord for BitString {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp_string(other)
    }
}
