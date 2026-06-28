use core::cmp::Ordering;

use crate::WORD_BITS;

use crate::BitStr;

mod inner;

impl<'bs> BitStr<'bs> {
    #[inline]
    pub fn cmp_str(&self, other: &BitStr<'bs>) -> Ordering {
        let hs_aligned = self.start % WORD_BITS == 0;
        let nd_aligned = other.start % WORD_BITS == 0;
        match (hs_aligned, nd_aligned) {
            (true, true) => self.cmp_inner::<true, true>(other),
            (true, false) => self.cmp_inner::<true, false>(other),
            (false, true) => self.cmp_inner::<false, true>(other),
            (false, false) => self.cmp_inner::<false, false>(other),
        }
    }

    #[inline]
    pub fn cmp_string(&self, other: &crate::BitString) -> Ordering {
        let o = other.as_bit_str();
        if self.start % WORD_BITS == 0 {
            self.cmp_inner::<true, true>(&o)
        } else {
            self.cmp_inner::<false, true>(&o)
        }
    }
}

impl PartialOrd for BitStr<'_> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp_str(other))
    }
}

impl Ord for BitStr<'_> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp_str(other)
    }
}

#[cfg(test)]
mod tests_for_ord;
