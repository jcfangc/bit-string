use core::cmp::Ordering;

use crate::BitString;

impl PartialOrd for BitString {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BitString {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_bit_str().cmp(&other.as_bit_str())
    }
}
